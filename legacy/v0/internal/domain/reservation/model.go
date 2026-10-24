package reservation

import "time"

type ReservationStatus string

const (
	StatusDRAFT          ReservationStatus = "DRAFT"
	StatusPENDINGPAYMENT ReservationStatus = "PENDING_PAYMENT"
	StatusCONFIRMED      ReservationStatus = "CONFIRMED"
	StatusCOMPLETED      ReservationStatus = "COMPLETED"
	StatusCANCELLED      ReservationStatus = "CANCELLED"
	StatusEXPIRED        ReservationStatus = "EXPIRED"
)

type Reservation struct {
	ID            string            `json:"id" db:"id"`
	ReferenceCode string            `json:"reference_code" db:"reference_code"`
	ResourceID    string            `json:"resource_id" db:"resource_id"`
	UserID        string            `json:"user_id" db:"user_id"`
	StartTime     time.Time         `json:"start_time" db:"start_time"`
	EndTime       time.Time         `json:"end_time" db:"end_time"`
	Status        ReservationStatus `json:"status" db:"status"`
	TotalAmount   float64           `json:"total_amount" db:"total_amount"`
	CreatedAt     time.Time         `json:"created_at" db:"created_at"`
	UpdatedAt     time.Time         `json:"updated_at" db:"updated_at"`
}
