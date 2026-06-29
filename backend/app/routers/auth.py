"""Auth router — login, change password, forgot password."""

from fastapi import APIRouter, Depends, HTTPException, Request, status
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.schemas.auth import (
    ChangePasswordRequest,
    ForgotPasswordRequest,
    LoginRequest,
    LoginResponse,
    RegisterRequest,
    UserResponse,
)
from app.services.auth import (
    authenticate_user,
    create_access_token,
    decode_access_token,
    hash_password,
    verify_password,
)
from app.models.user import User

router = APIRouter(prefix="/api/auth", tags=["Authentication"])


@router.post("/login", response_model=LoginResponse)
async def login(body: LoginRequest, request: Request, db: AsyncSession = Depends(get_db)):
    """Login and get JWT token."""
    user = await authenticate_user(db, body.username, body.password)

    if not user:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Username atau password tidak valid",
        )

    # Check if first login with temp password
    # TODO: implement first_login flag check

    token = create_access_token(user.id_user, user.role)

    # Log login in audit
    # TODO: create audit log entry

    return LoginResponse(
        access_token=token,
        role=user.role,
        expires_in=30 * 60,  # 30 minutes default
    )


@router.post("/register", status_code=201)
async def register(body: RegisterRequest, request: Request, db: AsyncSession = Depends(get_db)):
    """Register new guardian account."""
    from sqlalchemy import select
    
    # Check username uniqueness
    existing = await db.execute(select(User).where(User.username == body.username))
    if existing.scalar_one_or_none():
        raise HTTPException(
            status_code=status.HTTP_409_CONFLICT,
            detail="Username sudah digunakan",
        )
    
    # Check email uniqueness
    existing = await db.execute(select(User).where(User.email == body.email))
    if existing.scalar_one_or_none():
        raise HTTPException(
            status_code=status.HTTP_409_CONFLICT,
            detail="Email sudah terdaftar",
        )
    
    # Create user with guardian role
    pw_hash = hash_password(body.password)
    user = User(
        username=body.username,
        email=body.email,
        password_hash=pw_hash,
        role="guardian",
        is_active=True,
    )
    db.add(user)
    await db.flush()
    
    # Create guardian profile
    from app.models.guardian import Guardian
    from app.services.whatsapp import normalize_phone
    guardian = Guardian(
        id_user=user.id_user,
        full_name=body.full_name,
        phone_whatsapp=normalize_phone(body.phone),
        email=body.email,
        account_status="active",
    )
    db.add(guardian)
    await db.commit()
    
    return {
        "detail": "Registrasi berhasil. Silakan login.",
        "username": user.username,
        "role": user.role,
    }


@router.post("/change-password")
async def change_password(
    body: ChangePasswordRequest,
    request: Request,
    db: AsyncSession = Depends(get_db),
):
    """Change password (requires current password)."""
    # TODO: get current user from JWT token
    # For now, this is a placeholder
    return {"message": "Password berhasil diubah"}


@router.post("/forgot-password")
async def forgot_password(body: ForgotPasswordRequest, db: AsyncSession = Depends(get_db)):
    """Send password reset email."""
    # TODO: implement email reset flow
    return {"message": "Link reset telah dikirim ke email Anda"}


@router.get("/me", response_model=UserResponse)
async def get_current_user(request: Request):
    """Get current authenticated user info."""
    # TODO: extract user from JWT token
    # For now, return placeholder
    raise HTTPException(
        status_code=status.HTTP_401_UNAUTHORIZED,
        detail="Belum terautentikasi",
    )
