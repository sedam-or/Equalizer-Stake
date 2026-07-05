"""Teacher schemas — Pydantic models for teacher management."""

from uuid import UUID
from pydantic import BaseModel, Field


class TeacherCreate(BaseModel):
    """Admin creates teacher profile (User must already exist with role=teacher)."""
    id_user: UUID
    full_name: str = Field(..., min_length=2, max_length=255)
    instruments_specialization: list[str] = Field(..., min_length=1)
    employment_type: str = Field(..., pattern="^(tetap|freelance)$")
    max_students_per_session: int = Field(1, ge=1, le=10)


class TeacherUpdate(BaseModel):
    """Admin updates teacher profile."""
    full_name: str | None = Field(None, min_length=2, max_length=255)
    instruments_specialization: list[str] | None = None
    employment_type: str | None = Field(None, pattern="^(tetap|freelance)$")
    max_students_per_session: int | None = Field(None, ge=1, le=10)
    is_active: bool | None = None


class TeacherResponse(BaseModel):
    """Teacher profile response."""
    id_teacher: str
    id_user: str
    full_name: str
    instruments_specialization: list[str]
    employment_type: str
    max_students_per_session: int
    is_active: bool
    username: str | None = None
