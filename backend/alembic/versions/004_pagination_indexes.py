"""Add pagination performance indexes

Revision ID: 004_pagination_indexes
Revises: 003_draft_unique_constraint
Create Date: 2026-07-03
"""
from alembic import op
import sqlalchemy as sa

revision = "004_pagination_indexes"
down_revision = "003_draft_unique_constraint"
branch_labels = None
depends_on = None


def upgrade() -> None:
    # Students
    op.create_index("idx_students_created_at", "students", [sa.text("created_at DESC")])
    op.create_index("idx_students_instrument", "students", ["instrument"])
    op.create_index("idx_students_id_guardian", "students", ["id_guardian"])
    op.create_index("idx_students_deleted_at", "students", ["deleted_at"], postgresql_where=sa.text("deleted_at IS NULL"))

    # Sessions
    op.create_index("idx_sessions_session_date", "sessions", [sa.text("session_date DESC"), "start_time"])
    op.create_index("idx_sessions_id_teacher", "sessions", ["id_teacher"])
    op.create_index("idx_sessions_id_student", "sessions", ["id_student"])
    op.create_index("idx_sessions_status", "sessions", ["status"])

    # Attendances
    op.create_index("idx_attendances_created_at", "attendances", [sa.text("created_at DESC")])
    op.create_index("idx_attendances_id_teacher", "attendances", ["id_teacher"])
    op.create_index("idx_attendances_verification", "attendances", ["verification_status"])

    # Invoices
    op.create_index("idx_invoices_created_at", "invoices", [sa.text("created_at DESC")])
    op.create_index("idx_invoices_id_guardian", "invoices", ["id_guardian"])
    op.create_index("idx_invoices_status", "invoices", ["status"])

    # Payments
    op.create_index("idx_payments_created_at", "payments", [sa.text("created_at DESC")])
    op.create_index("idx_payments_is_verified", "payments", ["is_verified"])
    op.create_index("idx_payments_method", "payments", ["method"])

    # Notifications
    op.create_index("idx_notifications_created_at", "notifications", [sa.text("created_at DESC")])
    op.create_index("idx_notifications_recipient_id", "notifications", ["recipient_id"])


def downgrade() -> None:
    # Students
    op.drop_index("idx_students_created_at", "students")
    op.drop_index("idx_students_instrument", "students")
    op.drop_index("idx_students_id_guardian", "students")
    op.drop_index("idx_students_deleted_at", "students")

    # Sessions
    op.drop_index("idx_sessions_session_date", "sessions")
    op.drop_index("idx_sessions_id_teacher", "sessions")
    op.drop_index("idx_sessions_id_student", "sessions")
    op.drop_index("idx_sessions_status", "sessions")

    # Attendances
    op.drop_index("idx_attendances_created_at", "attendances")
    op.drop_index("idx_attendances_id_teacher", "attendances")
    op.drop_index("idx_attendances_verification", "attendances")

    # Invoices
    op.drop_index("idx_invoices_created_at", "invoices")
    op.drop_index("idx_invoices_id_guardian", "invoices")
    op.drop_index("idx_invoices_status", "invoices")

    # Payments
    op.drop_index("idx_payments_created_at", "payments")
    op.drop_index("idx_payments_is_verified", "payments")
    op.drop_index("idx_payments_method", "payments")

    # Notifications
    op.drop_index("idx_notifications_created_at", "notifications")
    op.drop_index("idx_notifications_recipient_id", "notifications")
