"""extend audit_action enum for auth events, make changed_by nullable, add details column

Revision ID: 002_auth_audit
Revises: 001_initial
Create Date: 2026-07-01
"""

from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa
from sqlalchemy.dialects import postgresql

revision: str = "002_auth_audit"
down_revision: Union[str, None] = "001_initial"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    # Extend audit_action enum with auth event values
    op.execute("ALTER TYPE audit_action ADD VALUE IF NOT EXISTS 'LOGIN_SUCCESS'")
    op.execute("ALTER TYPE audit_action ADD VALUE IF NOT EXISTS 'LOGIN_FAILED'")
    op.execute("ALTER TYPE audit_action ADD VALUE IF NOT EXISTS 'REGISTER'")
    op.execute("ALTER TYPE audit_action ADD VALUE IF NOT EXISTS 'PASSWORD_CHANGE'")
    op.execute("ALTER TYPE audit_action ADD VALUE IF NOT EXISTS 'ACCOUNT_LOCKED'")

    # Make changed_by nullable (for failed logins with unknown user)
    op.alter_column("audit_logs", "changed_by", nullable=True)

    # Add details column for auth event descriptions
    op.add_column("audit_logs", sa.Column("details", sa.Text(), nullable=True))


def downgrade() -> None:
    op.drop_column("audit_logs", "details")
    op.alter_column("audit_logs", "changed_by", nullable=False)
    # PostgreSQL does not support removing enum values; recreate type if needed
