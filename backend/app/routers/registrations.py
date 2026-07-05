"""Registration router — visitor registration + admin approval flow (T-066)."""
import os
import uuid

from datetime import date, datetime, timedelta, timezone
from uuid import UUID

from fastapi import APIRouter, Depends, File, Form, HTTPException, Query, UploadFile, status
from sqlalchemy import select, func
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.database import get_db
from app.middleware.auth import require_role
from app.models.guardian import Guardian
from app.models.registration import Registration
from app.models.student import Student
from app.models.user import User
from app.schemas.registration import RegistrationCreate, RegistrationReject
from app.services.admin_mgmt import generate_temp_password
from app.services.auth import hash_password
from app.services.audit import create_audit_log
from app.services.encryption import hash_pii

router = APIRouter(prefix="/api/v1/registrations", tags=["Registrations"])


# ── Public: visitor submits registration form ──────────────────────────

@router.post("", status_code=status.HTTP_201_CREATED)
async def submit_registration(
    full_name: str = Form(..., min_length=2, max_length=255),
    phone_whatsapp: str = Form(..., min_length=8, max_length=20),
    email: str = Form(..., min_length=5, max_length=255),
    address: str = Form(None),
    username: str = Form(..., min_length=3, max_length=50),
    password: str = Form(..., min_length=8, max_length=100),
    instrument: str = Form(None, max_length=50),
    level: str = Form(None, max_length=50),
    class_type: str = Form(None, max_length=50),
    schedule_id: str = Form(None),
    children: str = Form(None),  # JSON array of {full_name, instrument, level}
    payment_proof: UploadFile = File(None),
    db: AsyncSession = Depends(get_db),
):
    """Publik: buat akun + daftar kursus dalam satu request."""
    # Check username uniqueness
    existing_user = await db.execute(select(User).where(User.username == username))
    if existing_user.scalar_one_or_none():
        raise HTTPException(status_code=400, detail="Username sudah digunakan")
    
    # Check email uniqueness (via hash)
    from app.services.encryption import hash_pii
    email_hash_val = hash_pii(email)
    existing_email = await db.execute(select(User).where(User.email_hash == email_hash_val))
    if existing_email.scalar_one_or_none():
        raise HTTPException(status_code=400, detail="Email sudah terdaftar")
    
    # Create User
    user = User(
        username=username,
        email=email,
        password_hash=hash_password(password),
        role="guardian",
        email_hash=email_hash_val,
    )
    db.add(user)
    await db.flush()
    
    # Create Guardian
    guardian = Guardian(
        id_user=user.id_user,
        full_name=full_name,
        phone_whatsapp=phone_whatsapp,
        address=address,
    )
    db.add(guardian)
    await db.flush()
    
    # Save payment proof if provided
    proof_path = None
    if payment_proof and payment_proof.filename:
        upload_dir = "uploads/registration"
        os.makedirs(upload_dir, exist_ok=True)
        file_ext = os.path.splitext(payment_proof.filename)[1]
        file_name = f"{uuid.uuid4()}{file_ext}"
        file_path = os.path.join(upload_dir, file_name)
        file_content = await payment_proof.read()
        with open(file_path, "wb") as f:
            f.write(file_content)
        proof_path = file_path
    
    # Parse children data
    import json
    children_data = None
    if children:
        try:
            children_data = json.loads(children)
        except json.JSONDecodeError:
            raise HTTPException(status_code=400, detail="Format data anak tidak valid")

    # Create Registration
    reg = Registration(
        full_name=full_name,
        phone_whatsapp=phone_whatsapp,
        email=email,
        address=address,
        instrument=instrument,
        level=level,
        class_type=class_type,
        schedule_id=schedule_id,
        payment_proof_path=proof_path,
        children_data=children_data,
    )
    db.add(reg)
    await db.flush()
    
    return {
        "id_user": str(user.id_user),
        "id_guardian": str(guardian.id_guardian),
        "id_registration": str(reg.id_registration),
        "status": reg.status,
        "message": "Pendaftaran berhasil. Akun Anda telah dibuat. Menunggu verifikasi admin.",
    }


# ── Admin: list all registrations ─────────────────────────────────────

@router.get("")
async def list_registrations(
    status_filter: str | None = None,
    limit: int = Query(20, le=100),
    offset: int = Query(0, ge=0),
    user: User = Depends(require_role("owner", "admin")),
    db: AsyncSession = Depends(get_db),
):
    """Admin/Owner: lihat semua pendaftaran."""
    base = select(Registration).where(Registration.deleted_at.is_(None))
    if status_filter:
        base = base.where(Registration.status == status_filter)
    count_q = await db.execute(select(func.count()).select_from(base.subquery()))
    total = count_q.scalar()
    result = await db.execute(base.order_by(Registration.created_at.desc()).offset(offset).limit(limit))
    regs = result.scalars().all()
    return {
        "data": [
            {
                "id_registration": str(r.id_registration),
                "full_name": r.full_name,
                "phone_whatsapp": r.phone_whatsapp,
                "email": r.email,
                "address": r.address,
                "instrument": r.instrument,
                "level": r.level,
                "class_type": r.class_type,
                "status": r.status,
                "notes": r.notes,
                "reviewed_by": str(r.reviewed_by) if r.reviewed_by else None,
                "reviewed_at": r.reviewed_at.isoformat() if r.reviewed_at else None,
                "created_at": r.created_at.isoformat(),
            }
            for r in regs
        ],
        "total": total,
        "limit": limit,
        "offset": offset,
    }


# ── Admin: approve registration → auto-create user+guardian+student ───

@router.put("/{reg_id}/approve")
async def approve_registration(
    reg_id: str,
    user: User = Depends(require_role("owner", "admin")),
    db: AsyncSession = Depends(get_db),
):
    """Admin menyetujui pendaftaran. Auto-buat akun Guardian + Student."""
    reg = await _get_pending_reg(db, reg_id)

    # Check duplicate phone (username must be unique)
    existing_user = await db.execute(
        select(User).where(User.username == reg.phone_whatsapp)
    )
    if existing_user.scalar_one_or_none():
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Nomor WhatsApp sudah terdaftar sebagai akun pengguna.",
        )

    # 1. Create User (guardian role)
    temp_pw = generate_temp_password()
    now = datetime.now(timezone.utc)
    user_email = reg.email or f"{reg.phone_whatsapp}@bmdmusic.local"
    new_user = User(
        username=reg.phone_whatsapp,
        email=user_email,
        email_hash=hash_pii(user_email),
        password_hash=hash_password(temp_pw),
        role="guardian",
        is_active=True,
        first_login=True,
        is_temp_password=True,
        password_expires_at=now + timedelta(days=7),
    )
    db.add(new_user)
    await db.flush()

    # 2. Create Guardian
    guardian = Guardian(
        id_user=new_user.id_user,
        full_name=reg.full_name,
        phone_whatsapp=reg.phone_whatsapp,
        phone_hash=hash_pii(reg.phone_whatsapp),
        email=reg.email,
        address=reg.address,
    )
    db.add(guardian)
    await db.flush()

    # 3. Create Students from children_data (or single from registration)
    students_created = []
    if reg.children_data:
        for child in reg.children_data:
            student = Student(
                id_guardian=guardian.id_guardian,
                full_name=child.get("full_name", reg.full_name),
                instrument=child.get("instrument", reg.instrument),
                level=child.get("level", "pemula"),
                enrollment_status="active",
                enrollment_date=date.today(),
            )
            db.add(student)
            await db.flush()
            students_created.append(str(student.id_student))
    else:
        # Fallback: single student from registration data
        student = Student(
            id_guardian=guardian.id_guardian,
            full_name=reg.full_name,
            instrument=reg.instrument,
            level=reg.level,
            enrollment_status="active",
            enrollment_date=date.today(),
        )
        db.add(student)
        await db.flush()
        students_created.append(str(student.id_student))

    # 4. Update registration status
    reg.status = "approved"
    reg.reviewed_by = user.id_user
    reg.reviewed_at = now

    await create_audit_log(
        db,
        table_name="registrations",
        record_id=reg.id_registration,
        action="UPDATE",
        changed_by=user.id_user,
        before_state={"status": "pending"},
        after_state={
            "status": "approved",
            "user_created": str(new_user.id_user),
            "guardian_created": str(guardian.id_guardian),
            "students_created": students_created,
        },
    )
    
    # 5. Create notification for guardian
    from app.models.notification import Notification
    notif = Notification(
        id_user=new_user.id_user,
        category="registration_approved",
        title="Selamat Datang di BMD Musik!",
        message=f"Akun Anda sudah aktif. Username: {reg.phone_whatsapp}. Silakan login dan ganti password Anda.",
        channel="whatsapp",
        status="pending",
    )
    db.add(notif)
    
    # 6. Try to send email (graceful if SMTP not configured)
    try:
        from app.services.email import send_email_with_retry, render_template
        from app.core.config import get_settings
        settings = get_settings()
        if settings.SMTP_HOST and settings.SMTP_USER:
            subject, body = render_template("welcome", {
                "name": reg.full_name,
                "username": reg.phone_whatsapp,
                "temp_password": temp_pw,
                "login_url": "http://localhost:3000/login",
            })
            await send_email_with_retry(
                settings.SMTP_HOST, settings.SMTP_PORT,
                settings.SMTP_USER, settings.SMTP_PASSWORD,
                reg.email or f"{reg.phone_whatsapp}@bmdmusic.local",
                subject, body,
            )
    except Exception:
        pass  # Email not configured yet, skip silently
    
    await db.commit()

    return {
        "message": "Pendaftaran disetujui. Akun wali murid berhasil dibuat.",
        "registration_id": str(reg.id_registration),
        "user": {
            "id_user": str(new_user.id_user),
            "username": new_user.username,
            "temp_password": temp_pw,
        },
        "guardian_id": str(guardian.id_guardian),
        "student_id": str(student.id_student),
    }


# ── Admin: reject registration ────────────────────────────────────────

@router.put("/{reg_id}/reject")
async def reject_registration(
    reg_id: str,
    data: RegistrationReject,
    user: User = Depends(require_role("owner", "admin")),
    db: AsyncSession = Depends(get_db),
):
    """Admin menolak pendaftaran dengan alasan."""
    reg = await _get_pending_reg(db, reg_id)

    now = datetime.now(timezone.utc)
    reg.status = "rejected"
    reg.reviewed_by = user.id_user
    reg.reviewed_at = now
    reg.notes = data.notes

    await create_audit_log(
        db,
        table_name="registrations",
        record_id=reg.id_registration,
        action="UPDATE",
        changed_by=user.id_user,
        before_state={"status": "pending"},
        after_state={"status": "rejected", "notes": data.notes},
    )
    await db.commit()

    return {
        "message": "Pendaftaran ditolak.",
        "registration_id": str(reg.id_registration),
    }


# ── Helpers ────────────────────────────────────────────────────────────

async def _get_pending_reg(db: AsyncSession, reg_id: str) -> Registration:
    """Fetch registration by ID, must be pending."""
    try:
        rid = UUID(reg_id)
    except ValueError:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="ID pendaftaran tidak valid.",
        )

    result = await db.execute(
        select(Registration).where(Registration.id_registration == rid)
    )
    reg = result.scalar_one_or_none()
    if not reg:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Pendaftaran tidak ditemukan.",
        )
    if reg.status != "pending":
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=f"Pendaftaran sudah diproses (status: {reg.status}).",
        )
    return reg


@router.delete("/{reg_id}")
async def delete_registration(
    reg_id: str,
    user: User = Depends(require_role("owner", "admin")),
    db: AsyncSession = Depends(get_db),
):
    """Soft-delete registration."""
    result = await db.execute(
        select(Registration).where(Registration.id_registration == UUID(reg_id), Registration.deleted_at.is_(None))
    )
    reg = result.scalar_one_or_none()
    if not reg:
        raise HTTPException(status_code=404, detail="Pendaftaran tidak ditemukan")
    reg.deleted_at = datetime.now(timezone.utc)
    await db.commit()
    return {"detail": "Pendaftaran berhasil dihapus"}


# ── Public: available schedules for registration ─────────────────────

@router.get("/available-schedules")
async def get_available_schedules(
    instrument: str = Query(None),
    db: AsyncSession = Depends(get_db),
):
    """Publik: ambil jadwal yang tersedia untuk pendaftaran."""
    from app.models.session import Session
    from app.models.room import Room
    from app.models.teacher import Teacher
    
    # Get future sessions that are not cancelled
    now = datetime.now(timezone.utc)
    base = (
        select(Session)
        .where(Session.session_date >= now.date())
        .where(Session.status != "cancelled")
    )
    
    result = await db.execute(base)
    sessions = result.scalars().all()
    
    # Get all rooms and teachers
    rooms_result = await db.execute(select(Room).where(Room.is_available == True))
    rooms = {str(r.id_room): r for r in rooms_result.scalars().all()}
    
    teachers_result = await db.execute(select(Teacher))
    teachers = {str(t.id_teacher): t for t in teachers_result.scalars().all()}
    
    # Build available slots
    available = []
    for session in sessions:
        room = rooms.get(str(session.id_room))
        teacher = teachers.get(str(session.id_teacher))
        available.append({
            "id": str(session.id_session),
            "day": session.session_date.strftime("%A"),
            "date": session.session_date.strftime("%Y-%m-%d"),
            "start_time": session.start_time.strftime("%H:%M") if session.start_time else None,
            "end_time": session.end_time.strftime("%H:%M") if session.end_time else None,
            "room": room.room_name if room else "Unknown",
            "teacher": teacher.full_name if teacher else "Unknown",
        })
    
    return {"schedules": available}
