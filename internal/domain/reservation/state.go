package reservation

import "fmt"

var validTransitions = map[ReservationStatus][]ReservationStatus{
	StatusDRAFT:          {StatusPENDINGPAYMENT, StatusCANCELLED},
	StatusPENDINGPAYMENT: {StatusCONFIRMED, StatusCANCELLED, StatusEXPIRED},
	StatusCONFIRMED:      {StatusCOMPLETED, StatusCANCELLED},
}

var ErrInvalidTransition = fmt.Errorf("invalid status transition")

func CanTransition(from, to ReservationStatus) bool {
	targets, ok := validTransitions[from]
	if !ok {
		return false
	}
	for _, t := range targets {
		if t == to {
			return true
		}
	}
	return false
}
