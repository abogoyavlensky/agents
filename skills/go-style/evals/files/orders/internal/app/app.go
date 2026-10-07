// Package app is the composition root.
package app

import (
	"context"
	"database/sql"
	"errors"
	"fmt"
	"log/slog"
	"net/http"
	"os"
	"time"

	_ "github.com/jackc/pgx/v5/stdlib"

	"example.com/orders/internal/httpapi"
	"example.com/orders/internal/order"
	"example.com/orders/internal/sqlstore"
)

// Config is the application's configuration.
type Config struct {
	HTTPAddr        string
	DatabaseURL     string
	ShutdownTimeout time.Duration
}

// LoadConfig reads configuration from the environment.
func LoadConfig() (Config, error) {
	cfg := Config{
		HTTPAddr:        ":8080",
		DatabaseURL:     os.Getenv("DATABASE_URL"),
		ShutdownTimeout: 10 * time.Second,
	}
	if v := os.Getenv("HTTP_ADDR"); v != "" {
		cfg.HTTPAddr = v
	}
	if cfg.DatabaseURL == "" {
		return Config{}, errors.New("DATABASE_URL is required")
	}
	return cfg, nil
}

// App is the running system.
type App struct {
	cfg    Config
	logger *slog.Logger
	db     *sql.DB
	server *http.Server
}

// New builds the dependency graph.
func New(cfg Config, logger *slog.Logger) (*App, error) {
	db, err := sql.Open("pgx", cfg.DatabaseURL)
	if err != nil {
		return nil, fmt.Errorf("open database: %w", err)
	}

	orders := order.NewService(sqlstore.NewOrderStore(db), logger)
	handler := httpapi.NewHandler(orders, logger)

	server := &http.Server{Addr: cfg.HTTPAddr, Handler: handler}

	return &App{cfg: cfg, logger: logger, db: db, server: server}, nil
}

// Run serves until ctx is cancelled or the server fails, then shuts down.
func (a *App) Run(ctx context.Context) error {
	serveErr := make(chan error, 1)
	go func() {
		serveErr <- a.server.ListenAndServe()
	}()
	a.logger.InfoContext(ctx, "listening", "addr", a.cfg.HTTPAddr)

	var runErr error
	select {
	case <-ctx.Done():
	case err := <-serveErr:
		runErr = fmt.Errorf("serve http: %w", err)
	}

	shutdownCtx, cancel := context.WithTimeout(context.Background(), a.cfg.ShutdownTimeout)
	defer cancel()

	return errors.Join(runErr, a.close(shutdownCtx))
}

func (a *App) close(ctx context.Context) error {
	var errs []error
	if err := a.server.Shutdown(ctx); err != nil {
		errs = append(errs, fmt.Errorf("shutdown http: %w", err))
	}
	if err := a.db.Close(); err != nil {
		errs = append(errs, fmt.Errorf("close database: %w", err))
	}
	return errors.Join(errs...)
}
