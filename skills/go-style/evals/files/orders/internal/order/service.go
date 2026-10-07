package order

import (
	"context"
	"fmt"
	"log/slog"
)

// Store is what the service needs from storage.
type Store interface {
	Insert(ctx context.Context, in PlaceInput) (Order, error)
	Find(ctx context.Context, id ID) (Order, error)
}

// Service orchestrates order operations.
type Service struct {
	store  Store
	logger *slog.Logger
}

// NewService returns a Service backed by store.
func NewService(store Store, logger *slog.Logger) *Service {
	return &Service{store: store, logger: logger}
}

// Place validates input and stores a new pending order.
func (s *Service) Place(ctx context.Context, in PlaceInput) (Order, error) {
	if err := ValidatePlaceInput(in); err != nil {
		return Order{}, err
	}

	o, err := s.store.Insert(ctx, in)
	if err != nil {
		return Order{}, fmt.Errorf("insert order: %w", err)
	}

	s.logger.InfoContext(ctx, "order placed", "order_id", o.ID)
	return o, nil
}

// Get returns the order with the given ID.
func (s *Service) Get(ctx context.Context, id ID) (Order, error) {
	return s.store.Find(ctx, id)
}
