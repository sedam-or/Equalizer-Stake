"""Admin management schemas — request/response models."""

from uuid import UUID

from pydantic import BaseModel, EmailStr, Field


class CreateAdminRequest(BaseModel):
    """Request body for creating admin account."""
    username: str = Field(..., min_length=6, max_length=50)
    email: EmailStr
    password: str = Field(..., min_length=8, max_length=128)


class AdminResponse(BaseModel):
    """Admin account info (no sensitive data)."""
    id_user: UUID
    username: str
    email: str
    is_active: bool
    created_at: str

    model_config = {"from_attributes": True}


class DeleteAdminRequest(BaseModel):
    """Request body for deleting admin — requires username confirmation."""
    confirm_username: str = Field(..., min_length=1, max_length=50)


class AdminListResponse(BaseModel):
    """List of admin accounts."""
    admins: list[AdminResponse]
    total: int
