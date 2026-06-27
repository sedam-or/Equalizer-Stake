"""Auth schemas — request/response for authentication."""

from datetime import datetime
from uuid import UUID

from pydantic import BaseModel, EmailStr, Field


class LoginRequest(BaseModel):
    """Login request body."""
    username: str = Field(..., min_length=6, max_length=50)
    password: str = Field(..., min_length=6, max_length=128)


class LoginResponse(BaseModel):
    """Login response body."""
    access_token: str
    token_type: str = "bearer"
    role: str
    expires_in: int  # seconds


class TokenPayload(BaseModel):
    """JWT token payload."""
    sub: UUID
    role: str
    exp: datetime
    iat: datetime


class ChangePasswordRequest(BaseModel):
    """Change password request body."""
    current_password: str = Field(..., min_length=6)
    new_password: str = Field(..., min_length=8, max_length=128)


class ForgotPasswordRequest(BaseModel):
    """Forgot password request body."""
    email: EmailStr


class ResetPasswordRequest(BaseModel):
    """Reset password request body."""
    token: str
    new_password: str = Field(..., min_length=8, max_length=128)


class UserResponse(BaseModel):
    """User info response (no sensitive data)."""
    id_user: UUID
    username: str
    email: str
    role: str
    is_active: bool
    last_login_at: datetime | None = None
    created_at: datetime

    model_config = {"from_attributes": True}
