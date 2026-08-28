package resource

import "time"

type ResourceType string

const (
	ResourceTypeStudio    ResourceType = "STUDIO"
	ResourceTypeEquipment ResourceType = "EQUIPMENT"
)

type Resource struct {
	ID              string                 `json:"id" db:"id"`
	Name            string                 `json:"name" db:"name"`
	Slug            string                 `json:"slug" db:"slug"`
	Type            ResourceType           `json:"type" db:"type"`
	BasePricePerHour float64               `json:"base_price_per_hour" db:"base_price_per_hour"`
	IsActive        bool                   `json:"is_active" db:"is_active"`
	Metadata        map[string]interface{} `json:"metadata" db:"metadata"`
	CreatedAt       time.Time              `json:"created_at" db:"created_at"`
	UpdatedAt       time.Time              `json:"updated_at" db:"updated_at"`
}

type Schedule struct {
	ID           string `json:"id" db:"id"`
	ResourceID   string `json:"resource_id" db:"resource_id"`
	InstructorID string `json:"instructor_id" db:"instructor_id"`
	DayOfWeek    int    `json:"day_of_week" db:"day_of_week"`
	StartTime    string `json:"start_time" db:"start_time"`
	EndTime      string `json:"end_time" db:"end_time"`
	IsRecurring  bool   `json:"is_recurring" db:"is_recurring"`
}
