package order_test

import (
	"context"
	"errors"
	"io"
	"log/slog"
	"testing"
	"time"

	"example.com/orders/internal/order"
)

type fakeStore struct {
	orders map[order.ID]order.Order
	nextID order.ID
}

func newFakeStore() *fakeStore {
	return &fakeStore{orders: map[order.ID]order.Order{}}
}

func (f *fakeStore) Insert(_ context.Context, in order.PlaceInput) (order.Order, error) {
	f.nextID++
	o := order.Order{
		ID:         f.nextID,
		CustomerID: in.CustomerID,
		Status:     order.StatusPending,
		TotalCents: in.TotalCents,
		CreatedAt:  time.Now(),
	}
	f.orders[o.ID] = o
	return o, nil
}

func (f *fakeStore) Find(_ context.Context, id order.ID) (order.Order, error) {
	o, ok := f.orders[id]
	if !ok {
		return order.Order{}, order.ErrNotFound
	}
	return o, nil
}

func newTestService(store order.Store) *order.Service {
	return order.NewService(store, slog.New(slog.NewTextHandler(io.Discard, nil)))
}

func TestPlaceCreatesPendingOrder(t *testing.T) {
	svc := newTestService(newFakeStore())

	o, err := svc.Place(context.Background(), order.PlaceInput{CustomerID: "c1", TotalCents: 500})
	if err != nil {
		t.Fatalf("place: %v", err)
	}
	if o.Status != order.StatusPending {
		t.Errorf("status = %q, want %q", o.Status, order.StatusPending)
	}
}

func TestPlaceRejectsInvalidInput(t *testing.T) {
	tests := []struct {
		name string
		in   order.PlaceInput
	}{
		{"missing customer", order.PlaceInput{TotalCents: 500}},
		{"zero total", order.PlaceInput{CustomerID: "c1"}},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			svc := newTestService(newFakeStore())

			_, err := svc.Place(context.Background(), tt.in)
			if !errors.Is(err, order.ErrInvalidInput) {
				t.Fatalf("err = %v, want ErrInvalidInput", err)
			}
		})
	}
}
