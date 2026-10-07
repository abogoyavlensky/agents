// Package sqlstore implements storage on PostgreSQL via database/sql.
package sqlstore

import (
	"context"
	"database/sql"
	"errors"
	"fmt"

	"example.com/orders/internal/order"
)

// OrderStore stores orders. It satisfies order.Store.
type OrderStore struct {
	db *sql.DB
}

// NewOrderStore returns an OrderStore using db.
func NewOrderStore(db *sql.DB) *OrderStore {
	return &OrderStore{db: db}
}

const insertOrderSQL = `
	INSERT INTO orders (customer_id, total_cents)
	VALUES ($1, $2)
	RETURNING id, customer_id, status, total_cents, created_at`

// Insert stores a new pending order.
func (s *OrderStore) Insert(ctx context.Context, in order.PlaceInput) (order.Order, error) {
	var o order.Order

	err := s.db.QueryRowContext(ctx, insertOrderSQL, in.CustomerID, in.TotalCents).
		Scan(&o.ID, &o.CustomerID, &o.Status, &o.TotalCents, &o.CreatedAt)
	if err != nil {
		return order.Order{}, err
	}

	return o, nil
}

const findOrderSQL = `
	SELECT id, customer_id, status, total_cents, created_at
	FROM orders
	WHERE id = $1`

// Find returns the order with the given ID, or order.ErrNotFound.
func (s *OrderStore) Find(ctx context.Context, id order.ID) (order.Order, error) {
	var o order.Order

	err := s.db.QueryRowContext(ctx, findOrderSQL, id).
		Scan(&o.ID, &o.CustomerID, &o.Status, &o.TotalCents, &o.CreatedAt)
	if errors.Is(err, sql.ErrNoRows) {
		return order.Order{}, order.ErrNotFound
	}
	if err != nil {
		return order.Order{}, fmt.Errorf("find order %d: %w", id, err)
	}

	return o, nil
}
