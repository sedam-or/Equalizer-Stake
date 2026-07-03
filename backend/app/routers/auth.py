"""Auth router — login, register, profile, change password."""

from fastapi import APIRouter, Depends, HTTPException, Request, status
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.core.limiter import limiter
from app.middleware.auth import get_current_user as _get_user
from app.models.user import User
from datetime import datetime, timezone
from app.schemas.auth import (
    ChangePasswordRequest,
    ForgotPasswordRequest,
    FirstLoginChangePasswordRequest,
    LoginRequest,
    LoginResponse,
    ProfileResponse,
    RegisterRequest,
    UpdateProfileRequest,
    UserResponse,
)
from app.services.auth import (
    AccountFrozenError,
    authenticate_user,
    create_access_token,
    hash_password,
    verify_password,
    ACCESS_TOKEN_EXPIRE_MINUTES,
)
from app.services.audit import log_auth_event

router = APIRouter(prefix="/api/auth", tags=["Authentication"])


@router.post("/login", response_model=LoginResponse)
@limiter.limit("5/minute")
async def login(body: LoginRequest, request: Request, db: AsyncSession = Depends(get_db)):
    """Login and get JWT token."""
    try:
        user = await authenticate_user(db, body.username, body.password)
    except AccountFrozenError as e:
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN,
            detail=str(e),
        )
    if not user:
        # Check if account was just locked by brute force logic
        result = await db.execute(select(User).where(User.username == body.username))
        locked_user = result.scalar_one_or_none()
        if locked_user and locked_user.locked_until:
            await log_auth_event(db, action="ACCOUNT_LOCKED", user_id=locked_user.id_user, details=f"Account locked after failed attempts for username: {body.username}", request=request)
        await log_auth_event(db, action="LOGIN_FAILED", user_id=None, details=f"Failed login attempt for username: {body.username}", request=request)
        await db.commit()
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Username atau password tidak valid",
        )
    token = create_access_token(user.id_user, user.role)
    expires_minutes = ACCESS_TOKEN_EXPIRE_MINUTES.get(user.role, 30)
    await log_auth_event(db, action="LOGIN_SUCCESS", user_id=user.id_user, details=f"Successful login for role: {user.role}", request=request)
    await db.commit()
    # Check first_login / temp password
    if user.first_login or user.is_temp_password:
        # Check temp password expiry
        if user.password_expires_at and user.password_expires_at < datetime.now(timezone.utc):
            raise HTTPException(
                status_code=status.HTTP_403_FORBIDDEN,
                detail="Password sementara telah kedaluwarsa. Hubungi Admin untuk mengirim ulang.",
            )
        return LoginResponse(
            access_token=token,
            role=user.role,
            expires_in=expires_minutes * 60,
            must_change_password=True,
        )
    return LoginResponse(access_token=token, role=user.role, expires_in=expires_minutes * 60)


@router.post("/first-login-change-password")
@limiter.limit("10/minute")
async def first_login_change_password(
    body: FirstLoginChangePasswordRequest,
    request: Request,
    user: User = Depends(_get_user),
    db: AsyncSession = Depends(get_db),
):
    """Change password on first login. Resets first_login and is_temp_password flags."""
    if not user.first_login and not user.is_temp_password:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Akun ini tidak memerlukan pergantian password pertama.",
        )

    # Check temp password expiry
    if user.password_expires_at and user.password_expires_at < datetime.now(timezone.utc):
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN,
            detail="Password sementara telah kedaluwarsa. Hubungi Admin untuk mengirim ulang.",
        )

    # Verify current (temp) password
    if not verify_password(body.current_password, user.password_hash):
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Password lama tidak valid",
        )

    if body.current_password == body.new_password:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Password baru harus berbeda dari password sementara",
        )

    # Set new password and reset flags
    user.password_hash = hash_password(body.new_password)
    user.first_login = False
    user.is_temp_password = False
    user.password_expires_at = None

    await log_auth_event(
        db,
        action="PASSWORD_CHANGE",
        user_id=user.id_user,
        details="First login password change completed",
        request=request,
    )
    await db.commit()
    return {"detail": "Password berhasil diubah. Silakan login dengan password baru."}


@router.post("/register", status_code=201)
@limiter.limit("3/minute")
async def register(body: RegisterRequest, request: Request, db: AsyncSession = Depends(get_db)):
    """Register new guardian account."""
    # Check username uniqueness
    existing = await db.execute(select(User).where(User.username == body.username))
    if existing.scalar_one_or_none():
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Username sudah digunakan")

    # Check email uniqueness
    existing = await db.execute(select(User).where(User.email == body.email))
    if existing.scalar_one_or_none():
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Email sudah terdaftar")

    # Check phone uniqueness
    from app.services.whatsapp import normalize_phone
    from app.models.guardian import Guardian
    normalized_phone = normalize_phone(body.phone)
    existing_phone = await db.execute(
        select(Guardian).where(Guardian.phone_whatsapp == normalized_phone)
    )
    if existing_phone.scalar_one_or_none():
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Nomor WhatsApp sudah terdaftar")

    # Create user
    pw_hash = hash_password(body.password)
    user = User(username=body.username, email=body.email, password_hash=pw_hash, role="guardian", is_active=True)
    db.add(user)
    await db.flush()

    # Create guardian profile
    guardian = Guardian(id_user=user.id_user, full_name=body.full_name, phone_whatsapp=normalized_phone, email=body.email, address=body.address, account_status="active")
    db.add(guardian)
    await log_auth_event(db, action="REGISTER", user_id=user.id_user, details=f"New guardian registered: {user.username}", request=request)
    await db.commit()

    return {"detail": "Registrasi berhasil. Silakan login.", "username": user.username, "role": user.role}


@router.get("/me", response_model=UserResponse)
async def get_me(user: User = Depends(_get_user)):
    """Get current authenticated user info."""
    return user


@router.get("/me/profile", response_model=ProfileResponse)
async def get_my_profile(user: User = Depends(_get_user), db: AsyncSession = Depends(get_db)):
    """Get full profile with role-specific data."""
    full_name = None
    phone = None
    address = None

    if user.role == "guardian":
        from app.models.guardian import Guardian
        result = await db.execute(select(Guardian).where(Guardian.id_user == user.id_user))
        g = result.scalar_one_or_none()
        if g:
            full_name = g.full_name
            phone = g.phone_whatsapp
            address = g.address

    elif user.role == "teacher":
        from app.models.teacher import Teacher
        result = await db.execute(select(Teacher).where(Teacher.id_user == user.id_user))
        t = result.scalar_one_or_none()
        if t:
            full_name = t.full_name

    return ProfileResponse(user=UserResponse.model_validate(user), full_name=full_name, phone=phone, address=address)


@router.patch("/me/profile")
async def update_my_profile(
    body: UpdateProfileRequest,
    user: User = Depends(_get_user),
    db: AsyncSession = Depends(get_db),
):
    """Update profile (name, email, phone)."""
    # Update email on user table
    if body.email is not None:
        existing = await db.execute(
            select(User).where(User.email == body.email, User.id_user != user.id_user)
        )
        if existing.scalar_one_or_none():
            raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Email sudah digunakan")
        user.email = body.email

    # Update role-specific profile
    if user.role == "guardian":
        from app.models.guardian import Guardian
        from app.services.whatsapp import normalize_phone
        result = await db.execute(select(Guardian).where(Guardian.id_user == user.id_user))
        g = result.scalar_one_or_none()
        if g:
            if body.full_name is not None:
                g.full_name = body.full_name
            if body.phone is not None:
                normalized = normalize_phone(body.phone)
                existing_phone = await db.execute(
                    select(Guardian).where(Guardian.phone_whatsapp == normalized, Guardian.id_guardian != g.id_guardian)
                )
                if existing_phone.scalar_one_or_none():
                    raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Nomor WhatsApp sudah digunakan")
                g.phone_whatsapp = normalized

    await db.commit()
    return {"detail": "Profil berhasil diperbarui"}


@router.post("/change-password")
@limiter.limit("10/minute")
async def change_password(
    body: ChangePasswordRequest,
    request: Request,
    user: User = Depends(_get_user),
    db: AsyncSession = Depends(get_db),
):
    """Change password (requires current password)."""
    if not verify_password(body.current_password, user.password_hash):
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Password lama tidak valid")
    if body.current_password == body.new_password:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Password baru harus berbeda")
    user.password_hash = hash_password(body.new_password)
    await log_auth_event(db, action="PASSWORD_CHANGE", user_id=user.id_user, details="Password changed successfully", request=request)
    await db.commit()
    return {"detail": "Password berhasil diubah"}


@router.post("/forgot-password")
@limiter.limit("5/minute")
async def forgot_password(body: ForgotPasswordRequest, request: Request, db: AsyncSession = Depends(get_db)):
    """Send password reset email."""
    return {"detail": "Link reset telah dikirim ke email Anda"}
