"""initial schema — 11 entities

Revision ID: 001_initial
Revises: 
Create Date: 2026-06-27
"""

from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa
from sqlalchemy.dialects import postgresql

revision: str = "001_initial"
down_revision: Union[str, None] = None
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    """Create all 11 tables."""
    
    # Enum types
    user_role = postgresql.ENUM("owner", "admin", "teacher", "guardian", name="user_role", create_type=False)
    account_status = postgresql.ENUM("active", "frozen", "inactive", "pending_deletion", name="account_status", create_type=False)
    instrument_type = postgresql.ENUM("biola", "piano", "gitar", "drum", "keyboard", "flute", "saxophone", "vokal", "ensemble", name="instrument_type", create_type=False)
    student_level = postgresql.ENUM("pemula", "dasar", "menengah", "lanjut", name="student_level", create_type=False)
    enrollment_status = postgresql.ENUM("active", "cuti", "inactive", "trial", name="enrollment_status", create_type=False)
    employment_type = postgresql.ENUM("tetap", "freelance", name="employment_type", create_type=False)
    room_type = postgresql.ENUM("kursus", "rehearsal", "recording", name="room_type", create_type=False)
    session_type = postgresql.ENUM("reguler", "teori", "rehearsal", "recording", "kompensasi", name="session_type", create_type=False)
    session_status = postgresql.ENUM("draft", "pending", "locked", "completed", "cancelled", "rescheduled", name="session_status", create_type=False)
    invoice_status = postgresql.ENUM("draft", "issued", "paid", "overdue", "cancelled", name="invoice_status", create_type=False)
    payment_method = postgresql.ENUM("transfer", "cash", "dp", name="payment_method", create_type=False)
    verification_status = postgresql.ENUM("verified", "pending_review", "rejected", name="verification_status", create_type=False)
    notif_channel = postgresql.ENUM("whatsapp", "email", "sms", name="notif_channel", create_type=False)
    notif_category = postgresql.ENUM("critical", "non_critical", name="notif_category", create_type=False)
    notif_status = postgresql.ENUM("pending", "sent", "failed", "retrying", name="notif_status", create_type=False)
    audit_action = postgresql.ENUM("INSERT", "UPDATE", "DELETE", name="audit_action", create_type=False)
    
    for e in [user_role, account_status, instrument_type, student_level, enrollment_status,
              employment_type, room_type, session_type, session_status, invoice_status,
              payment_method, verification_status, notif_channel, notif_category, notif_status, audit_action]:
        e.create(op.get_bind(), checkfirst=True)
    
    # 1. Users
    op.create_table(
        "users",
        sa.Column("id_user", postgresql.UUID(as_uuid=True), primary_key=True, server_default=sa.text("gen_random_uuid()")),
        sa.Column("username", sa.String(50), unique=True, nullable=False),
        sa.Column("email", sa.String(255), unique=True, nullable=False),
        sa.Column("password_hash", sa.String(255), nullable=False),
        sa.Column("role", user_role, nullable=False),
        sa.Column("is_active", sa.Boolean, server_default="true", nullable=False),
        sa.Column("last_login_at", sa.DateTime(timezone=True), nullable=True),
        sa.Column("refresh_token", sa.Text, nullable=True),
        sa.Column("token_expires_at", sa.DateTime(timezone=True), nullable=True),
        sa.Column("failed_login_attempts", sa.Integer, server_default="0", nullable=False),
        sa.Column("locked_until", sa.DateTime(timezone=True), nullable=True),
        sa.Column("created_at", sa.DateTime(timezone=True), server_default=sa.func.now(), nullable=False),
        sa.Column("updated_at", sa.DateTime(timezone=True), server_default=sa.func.now(), nullable=False),
        sa.Column("deleted_at", sa.DateTime(timezone=True), nullable=True),
    )
    
    # 2. Guardians
    op.create_table(
        "guardians",
        sa.Column("id_guardian", postgresql.UUID(as_uuid=True), primary_key=True, server_default=sa.text("gen_random_uuid()")),
        sa.Column("id_user", postgresql.UUID(as_uuid=True), sa.ForeignKey("users.id_user"), unique=True, nullable=False),
        sa.Column("full_name", sa.Text, nullable=False),
        sa.Column("phone_whatsapp", sa.String(20), unique=True, nullable=False),
        sa.Column("email", sa.String(255), nullable=True),
        sa.Column("address", sa.Text, nullable=True),
        sa.Column("account_status", account_status, nullable=False, server_default="active"),
        sa.Column("frozen_at", sa.DateTime(timezone=True), nullable=True),
        sa.Column("frozen_reason", sa.Text, nullable=True),
        sa.Column("created_at", sa.DateTime(timezone=True), server_default=sa.func.now(), nullable=False),
        sa.Column("updated_at", sa.DateTime(timezone=True), server_default=sa.func.now(), nullable=False),
        sa.Column("deleted_at", sa.DateTime(timezone=True), nullable=True),
    )
    
    # 3. Students
    op.create_table(
        "students",
        sa.Column("id_student", postgresql.UUID(as_uuid=True), primary_key=True, server_default=sa.text("gen_random_uuid()")),
        sa.Column("id_guardian", postgresql.UUID(as_uuid=True), sa.ForeignKey("guardians.id_guardian"), nullable=False),
        sa.Column("full_name", sa.Text, nullable=False),
        sa.Column("date_of_birth", sa.Date, nullable=True),
        sa.Column("instrument", instrument_type, nullable=False),
        sa.Column("level", student_level, nullable=False, server_default="pemula"),
        sa.Column("enrollment_status", enrollment_status, nullable=False, server_default="active"),
        sa.Column("enrollment_date", sa.Date, nullable=False),
        sa.Column("trial_end_date", sa.Date, nullable=True),
        sa.Column("eligible_for_exam", sa.Boolean, server_default="false", nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), server_default=sa.func.now(), nullable=False),
        sa.Column("updated_at", sa.DateTime(timezone=True), server_default=sa.func.now(), nullable=False),
        sa.Column("deleted_at", sa.DateTime(timezone=True), nullable=True),
    )
    op.create_index("ix_students_id_guardian", "students", ["id_guardian"])
    
    # 4. Teachers
    op.create_table(
        "teachers",
        sa.Column("id_teacher", postgresql.UUID(as_uuid=True), primary_key=True, server_default=sa.text("gen_random_uuid()")),
        sa.Column("id_user", postgresql.UUID(as_uuid=True), sa.ForeignKey("users.id_user"), unique=True, nullable=False),
        sa.Column("full_name", sa.String(255), nullable=False),
        sa.Column("instruments_specialization", postgresql.JSONB, nullable=False),
        sa.Column("employment_type", employment_type, nullable=False),
        sa.Column("max_students_per_session", sa.Integer, server_default="1", nullable=False),
        sa.Column("is_active", sa.Boolean, server_default="true", nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), server_default=sa.func.now(), nullable=False),
        sa.Column("updated_at", sa.DateTime(timezone=True), server_default=sa.func.now(), nullable=False),
        sa.Column("deleted_at", sa.DateTime(timezone=True), nullable=True),
    )
    
    # 5. Rooms
    op.create_table(
        "rooms",
        sa.Column("id_room", postgresql.UUID(as_uuid=True), primary_key=True, server_default=sa.text("gen_random_uuid()")),
        sa.Column("room_name", sa.String(100), unique=True, nullable=False),
        sa.Column("room_type", room_type, nullable=False),
        sa.Column("capacity", sa.Integer, server_default="1", nullable=False),
        sa.Column("is_available", sa.Boolean, server_default="true", nullable=False),
        sa.Column("location_notes", sa.Text, nullable=True),
        sa.Column("created_at", sa.DateTime(timezone=True), server_default=sa.func.now(), nullable=False),
        sa.Column("updated_at", sa.DateTime(timezone=True), server_default=sa.func.now(), nullable=False),
    )
    
    # 6. Sessions
    op.create_table(
        "sessions",
        sa.Column("id_session", postgresql.UUID(as_uuid=True), primary_key=True, server_default=sa.text("gen_random_uuid()")),
        sa.Column("id_teacher", postgresql.UUID(as_uuid=True), sa.ForeignKey("teachers.id_teacher"), nullable=False),
        sa.Column("id_student", postgresql.UUID(as_uuid=True), sa.ForeignKey("students.id_student"), nullable=False),
        sa.Column("id_room", postgresql.UUID(as_uuid=True), sa.ForeignKey("rooms.id_room"), nullable=False),
        sa.Column("session_date", sa.Date, nullable=False),
        sa.Column("start_time", sa.Time, nullable=False),
        sa.Column("end_time", sa.Time, nullable=False),
        sa.Column("session_type", session_type, nullable=False, server_default="reguler"),
        sa.Column("status", session_status, nullable=False, server_default="draft"),
        sa.Column("rescheduled_to", postgresql.UUID(as_uuid=True), sa.ForeignKey("sessions.id_session"), nullable=True),
        sa.Column("cancellation_reason", sa.Text, nullable=True),
        sa.Column("created_at", sa.DateTime(timezone=True), server_default=sa.func.now(), nullable=False),
        sa.Column("updated_at", sa.DateTime(timezone=True), server_default=sa.func.now(), nullable=False),
    )
    op.create_index("ix_sessions_teacher_date", "sessions", ["id_teacher", "session_date"])
    op.create_index("ix_sessions_student_date", "sessions", ["id_student", "session_date"])
    op.create_index("ix_sessions_room_date", "sessions", ["id_room", "session_date"])
    
    # 7. Invoices
    op.create_table(
        "invoices",
        sa.Column("id_invoice", postgresql.UUID(as_uuid=True), primary_key=True, server_default=sa.text("gen_random_uuid()")),
        sa.Column("id_guardian", postgresql.UUID(as_uuid=True), sa.ForeignKey("guardians.id_guardian"), nullable=False),
        sa.Column("unique_code", sa.CHAR(3), nullable=False),
        sa.Column("billing_period", sa.String(7), nullable=False),
        sa.Column("total_amount", sa.Numeric(12, 2), nullable=False),
        sa.Column("due_date", sa.Date, nullable=False),
        sa.Column("status", invoice_status, nullable=False, server_default="draft"),
        sa.Column("issued_at", sa.DateTime(timezone=True), nullable=True),
        sa.Column("paid_at", sa.DateTime(timezone=True), nullable=True),
        sa.Column("notes", sa.Text, nullable=True),
        sa.Column("created_at", sa.DateTime(timezone=True), server_default=sa.func.now(), nullable=False),
        sa.Column("updated_at", sa.DateTime(timezone=True), server_default=sa.func.now(), nullable=False),
    )
    op.create_index("ix_invoices_guardian_period", "invoices", ["id_guardian", "billing_period"], unique=True)
    
    # 8. Payments
    op.create_table(
        "payments",
        sa.Column("id_payment", postgresql.UUID(as_uuid=True), primary_key=True, server_default=sa.text("gen_random_uuid()")),
        sa.Column("id_invoice", postgresql.UUID(as_uuid=True), sa.ForeignKey("invoices.id_invoice"), nullable=False),
        sa.Column("amount", sa.Numeric(12, 2), nullable=False),
        sa.Column("method", payment_method, nullable=False),
        sa.Column("payment_date", sa.DateTime(timezone=True), nullable=False),
        sa.Column("proof_url", sa.Text, nullable=True),
        sa.Column("unique_code_used", sa.CHAR(3), nullable=False),
        sa.Column("is_verified", sa.Boolean, server_default="false", nullable=False),
        sa.Column("verified_by", postgresql.UUID(as_uuid=True), sa.ForeignKey("users.id_user"), nullable=True),
        sa.Column("verified_at", sa.DateTime(timezone=True), nullable=True),
        sa.Column("notes", sa.Text, nullable=True),
        sa.Column("created_at", sa.DateTime(timezone=True), server_default=sa.func.now(), nullable=False),
        sa.Column("updated_at", sa.DateTime(timezone=True), server_default=sa.func.now(), nullable=False),
    )
    
    # 9. Attendances
    op.create_table(
        "attendances",
        sa.Column("id_attendance", postgresql.UUID(as_uuid=True), primary_key=True, server_default=sa.text("gen_random_uuid()")),
        sa.Column("id_session", postgresql.UUID(as_uuid=True), sa.ForeignKey("sessions.id_session"), unique=True, nullable=False),
        sa.Column("id_teacher", postgresql.UUID(as_uuid=True), sa.ForeignKey("teachers.id_teacher"), nullable=False),
        sa.Column("photo_url", sa.Text, nullable=False),
        sa.Column("gps_lat", sa.Float, nullable=False),
        sa.Column("gps_lng", sa.Float, nullable=False),
        sa.Column("gps_distance_m", sa.Float, nullable=False),
        sa.Column("face_valid", sa.Boolean, nullable=False),
        sa.Column("face_confidence", sa.Float, nullable=True),
        sa.Column("client_timestamp", sa.DateTime(timezone=True), nullable=False),
        sa.Column("server_timestamp", sa.DateTime(timezone=True), server_default=sa.func.now(), nullable=False),
        sa.Column("verification_status", verification_status, nullable=False, server_default="pending_review"),
        sa.Column("rejection_reason", sa.Text, nullable=True),
        sa.Column("reviewed_by", postgresql.UUID(as_uuid=True), sa.ForeignKey("users.id_user"), nullable=True),
        sa.Column("reviewed_at", sa.DateTime(timezone=True), nullable=True),
        sa.Column("is_offline_submission", sa.Boolean, server_default="false", nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), server_default=sa.func.now(), nullable=False),
        sa.Column("updated_at", sa.DateTime(timezone=True), server_default=sa.func.now(), nullable=False),
    )
    
    # 10. Notifications
    op.create_table(
        "notifications",
        sa.Column("id_notification", postgresql.UUID(as_uuid=True), primary_key=True, server_default=sa.text("gen_random_uuid()")),
        sa.Column("recipient_id", postgresql.UUID(as_uuid=True), sa.ForeignKey("users.id_user"), nullable=False),
        sa.Column("channel", notif_channel, nullable=False),
        sa.Column("category", notif_category, nullable=False),
        sa.Column("subject", sa.String(255), nullable=True),
        sa.Column("body", sa.Text, nullable=False),
        sa.Column("status", notif_status, nullable=False, server_default="pending"),
        sa.Column("retry_count", sa.Integer, server_default="0", nullable=False),
        sa.Column("error_message", sa.Text, nullable=True),
        sa.Column("sent_at", sa.DateTime(timezone=True), nullable=True),
        sa.Column("related_entity_type", sa.String(50), nullable=True),
        sa.Column("related_entity_id", postgresql.UUID(as_uuid=True), nullable=True),
        sa.Column("created_at", sa.DateTime(timezone=True), server_default=sa.func.now(), nullable=False),
    )
    op.create_index("ix_notifications_recipient_status", "notifications", ["recipient_id", "status"])
    
    # 11. AuditLogs
    op.create_table(
        "audit_logs",
        sa.Column("id_audit", postgresql.UUID(as_uuid=True), primary_key=True, server_default=sa.text("gen_random_uuid()")),
        sa.Column("table_name", sa.Text, nullable=False),
        sa.Column("record_id", postgresql.UUID(as_uuid=True), nullable=False),
        sa.Column("action", audit_action, nullable=False),
        sa.Column("before_state", postgresql.JSONB, nullable=True),
        sa.Column("after_state", postgresql.JSONB, nullable=True),
        sa.Column("changed_by", postgresql.UUID(as_uuid=True), sa.ForeignKey("users.id_user"), nullable=False),
        sa.Column("ip_address", postgresql.INET, nullable=True),
        sa.Column("user_agent", sa.Text, nullable=True),
        sa.Column("created_at", sa.DateTime(timezone=True), server_default=sa.func.now(), nullable=False),
    )
    op.create_index("ix_audit_logs_table_record", "audit_logs", ["table_name", "record_id"])
    op.create_index("ix_audit_logs_changed_by", "audit_logs", ["changed_by"])


def downgrade() -> None:
    """Drop all 11 tables."""
    op.drop_table("audit_logs")
    op.drop_table("notifications")
    op.drop_table("attendances")
    op.drop_table("payments")
    op.drop_table("invoices")
    op.drop_table("sessions")
    op.drop_table("rooms")
    op.drop_table("teachers")
    op.drop_table("students")
    op.drop_table("guardians")
    op.drop_table("users")
    
    for name in ["audit_action", "notif_status", "notif_category", "notif_channel",
                  "verification_status", "payment_method", "invoice_status", "session_status",
                  "session_type", "room_type", "employment_type", "enrollment_status",
                  "student_level", "instrument_type", "account_status", "user_role"]:
        op.execute(f"DROP TYPE IF EXISTS {name}")
