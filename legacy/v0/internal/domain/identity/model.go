package identity

import "time"

type UserRole string

const (
	RoleSuperAdmin      UserRole = "SUPER_ADMIN"
	RoleResourceManager UserRole = "RESOURCE_MANAGER"
	RoleInstructor      UserRole = "INSTRUCTOR"
	RoleMember          UserRole = "MEMBER"
)

type User struct {
	ID           string    `json:"id" db:"id"`
	Email        string    `json:"email" db:"email"`
	PasswordHash string    `json:"-" db:"password_hash"`
	Name         string    `json:"name" db:"name"`
	Role         UserRole  `json:"role" db:"role"`
	IsActive     bool      `json:"is_active" db:"is_active"`
	CreatedAt    time.Time `json:"created_at" db:"created_at"`
	UpdatedAt    time.Time `json:"updated_at" db:"updated_at"`
}
