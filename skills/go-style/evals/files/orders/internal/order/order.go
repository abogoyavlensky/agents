// Package order holds the order domain.
package order

import (
	"errors"
	"time"
)

// ID identifies an order.
type ID int64

// Status is the lifecycle state of an order.
type Status string

const (
	StatusPending Status = "pending"
	StatusPaid    Status = "paid"
	StatusShipped Status = "shipped"
)

// Order is a customer order.
type Order struct {
	ID         ID        `json:"id"`
	CustomerID string    `json:"customer_id"`
	Status     Status    `json:"status"`
	TotalCents int64     `json:"total_cents"`
	CreatedAt  time.Time `json:"created_at"`
}

// PlaceInput is the data needed to place an order.
type PlaceInput struct {
	CustomerID string `json:"customer_id"`
	TotalCents int64  `json:"total_cents"`
}

// ErrNotFound is returned when an order does not exist.
var ErrNotFound = errors.New("order not found")

// ErrInvalidInput is returned when input fails validation.
var ErrInvalidInput = errors.New("invalid input")

// ValidatePlaceInput checks input for a new order.
func ValidatePlaceInput(in PlaceInput) error {
	if in.CustomerID == "" {
		return errors.Join(ErrInvalidInput, errors.New("customer_id is required"))
	}
	if in.TotalCents <= 0 {
		return errors.Join(ErrInvalidInput, errors.New("total_cents must be positive"))
	}
	return nil
}
