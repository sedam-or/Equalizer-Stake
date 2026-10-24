"""add delivered and fallback to notif_status enum

Revision ID: 003_notif_enhancements
Revises: 002_auth_audit
Create Date: 2026-07-01
"""

from typing import Sequence, Union

from alembic import op

revision: str = "003_notif_enhancements"
down_revision: Union[str, None] = "002_auth_audit"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.execute("ALTER TYPE notif_status ADD VALUE IF NOT EXISTS 'delivered'")
    op.execute("ALTER TYPE notif_status ADD VALUE IF NOT EXISTS 'fallback'")


def downgrade() -> None:
    # PostgreSQL does not support removing enum values directly.
    # Recreate enum without the new values if downgrade is needed.
    op.execute("ALTER TABLE notifications ALTER COLUMN status TYPE TEXT")
    op.execute("DROP TYPE notif_status")
    op.execute("CREATE TYPE notif_status AS ENUM ('pending', 'sent', 'failed', 'retrying')")
    op.execute("ALTER TABLE notifications ALTER COLUMN status TYPE notif_status USING status::notif_status")
