"""Auth schemas — request/response models for authentication."""

from datetime import datetime
from uuid import UUID

from pydantic import BaseModel, EmailStr, Field


class LoginRequest(BaseModel):
    """Login request body."""
    username: str
    password: str


class LoginResponse(BaseModel):
    """Login response with JWT token."""
    access_token: str
    token_type: str = "bearer"
    role: str
    expires_in: int  # seconds


class RegisterRequest(BaseModel):
    """Guardian registration request."""
    username: str = Field(min_length=6, max_length=50)
    email: EmailStr
    password: str = Field(min_length=8)
    full_name: str = Field(min_length=1, max_length=255)
    phone: str = Field(min_length=10, max_length=20)


class ChangePasswordRequest(BaseModel):
    """Change password request."""
    current_password: str
    new_password: str = Field(min_length=8)


class ForgotPasswordRequest(BaseModel):
    """Forgot password request."""
    email: EmailStr


class UpdateProfileRequest(BaseModel):
    """Update profile request (name, email, phone)."""
    full_name: str | None = None
    email: EmailStr | None = None
    phone: str | None = None


class UserResponse(BaseModel):
    """User info response (no sensitive data)."""
    id_user: UUID
    username: str
    email: str
    role: str
    is_active: bool
    last_login_at: datetime | None = None
    created_at: datetime | None = None

    model_config = {"from_attributes": True}


class ProfileResponse(BaseModel):
    """Full profile response with role-specific data."""
    user: UserResponse
    full_name: str | None = None
    phone: str | None = None
    address: str | None = None

    model_config = {"from_attributes": True}
