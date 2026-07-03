"""add composite unique constraint on drafts (id_user, form_key)

Revision ID: 003_draft_unique
Revises: 002_auth_audit
Create Date: 2026-07-01
"""

from typing import Sequence, Union

from alembic import op

revision: str = "003_draft_unique"
down_revision: Union[str, None] = "002_auth_audit"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.create_unique_constraint(
        "uq_user_form_key",
        "drafts",
        ["id_user", "form_key"],
    )


def downgrade() -> None:
    op.drop_constraint("uq_user_form_key", "drafts", type_="unique")
