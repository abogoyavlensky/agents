# Go style guide: rationale and worked examples

The reasoning behind the rules in `SKILL.md`, with examples. The code here is
taken from one small module (`example.com/shop`, Go 1.26, pgx, testify) that
builds, passes `go vet`, and passes `go test -race`. It's meant to be copied.
For older Go versions, see the notes next to each example.

## Contents

1. [Intent](#1-intent)
2. [Data flow and values](#2-data-flow-and-values)
3. [Project shape and naming](#3-project-shape-and-naming)
4. [Domain package: values, errors, service](#4-domain-package-values-errors-service)
5. [Storage](#5-storage)
6. [HTTP](#6-http)
7. [Composition root, lifecycle, and main](#7-composition-root-lifecycle-and-main)
8. [Concurrency, workers, and stateful types](#8-concurrency-workers-and-stateful-types)
9. [Errors and logging](#9-errors-and-logging)
10. [Tests](#10-tests)
11. [Abstractions that must earn their place](#11-abstractions-that-must-earn-their-place)
12. [Deployment shape](#12-deployment-shape)

---

## 1. Intent

The aim isn't Clojure written in Go. It's idiomatic Go that keeps the parts of
Clojure systems that make them pleasant: explicit data flow, values treated as
immutable by convention, functions over data, and a system graph that's
visible in code with a clear lifecycle (as with Integrant: config → build
graph → start → run → stop).

Think of an application as a graph of ordinary values and resources:

```text
                 Config
                   │
        ┌──────────┴──────────┐
        ↓                     ↓
    Database               Logger
        │                     │
        └──────────┬──────────┘
                   ↓
               UserStore
                   ↓
              user.Service
                   ↓
               HTTP API
                   ↓
                Server
```

Construction is explicit and so is ownership. Data flows through functions,
mutable state is contained, and infrastructure sits at the edges. No hidden
container drives the graph.

**Decision rule.** When two designs are close, choose the one with fewer
hidden rules. Prefer explicit wiring, explicit data, explicit errors, and an
explicit lifecycle over implicit registration, implicit mutation, implicit
dependencies, and implicit control flow. Explicit doesn't mean verbose,
though. A small helper that removes real repetition and keeps the flow
visible is a good thing.

**Why this suits agents.** Agents produce repetitive code cheaply, and humans
still have to review it. Prefer code that the compiler, `go vet`, an LSP,
grep, and a reviewer can all follow: direct calls, concrete types, field
access, explicit registration. Avoid reflection, runtime discovery, and
abstractions whose only gain is fewer lines for the agent to write.

---

## 2. Data flow and values

An application operation should read top to bottom and show its shape:

```text
input → normalize → validate → construct → persist → result
```

See `Service.Register` in section 4. Each step is a function over values, and
the only side effect, the store call, can be seen at the call site.

**Values by default.** Pass small domain structs by value and return them.
Taking a struct by value and returning a modified copy is the Go version of
`assoc`:

```go
func NormalizeRegisterInput(in RegisterInput) RegisterInput {
	in.Email = strings.ToLower(strings.TrimSpace(in.Email))
	in.Name = strings.TrimSpace(in.Name)
	return in
}
```

This mutates its own copy, and from the outside it behaves as a pure
transformation. That's fine. Immutability here is a way to reason about code,
not a purity contest.

Use a pointer when identity matters, when the type owns a resource or a lock,
when copying would be wrong or expensive, or when the type is deliberately
stateful (`*Service`, `*Counter`, `*pgxpool.Pool`).

**Don't mutate what the caller owns.** Slices and maps share their backing
storage, so a function that writes into a slice argument changes the caller's
data:

```go
// NormalizeTags returns lowercased copies of tags; the input is left untouched.
func NormalizeTags(tags []string) []string {
	out := make([]string, len(tags))
	for i, tag := range tags {
		out[i] = strings.ToLower(tag)
	}
	return out
}
```

Building a slice with `append` inside a function is normal Go and needs no
special treatment. Mutate in place only on a measured hot path, and make the
function's name and doc comment say that it mutates.

**Side effects are visible.** From the call site, a reader should be able to
tell which calls touch the database, network, files, or shared state. Package
names and naming usually make this clear: `user.ValidateRegisterInput`, then
`store.Insert`, then `mailer.Send`. Avoid object graphs where an innocent
method triggers I/O through a callback.

---

## 3. Project shape and naming

Organize packages around domains and real boundaries, not architectural
layers:

```text
cmd/
  shop/
    main.go          # boring: signals, config, app.New, Run
internal/
  app/               # composition root: config + wiring + lifecycle
    app.go
    config.go
  user/              # domain: values, rules, service, consumer interfaces
    user.go
    service.go
  analytics/
    worker.go
  postgres/          # storage boundary, implements domain interfaces
    user.go
  httpapi/           # HTTP boundary
    handler.go
web/                 # frontend, if embedded
```

Avoid `models/ services/ repositories/ controllers/`, `utils`, `common`,
`helpers`, and `pkg/`. A `utils` package means the code has no home yet.
`email.Normalize` and `httpapi.writeJSON` tell the reader something, and
`utils.Normalize` doesn't. Infrastructure packages (`postgres`, `httpapi`,
`app`) are fine because they're real boundaries. Don't add a layer just
because a diagram has one.

**Naming.** Callers always see the package name, so don't repeat it:
`user.Service`, `user.NewService`, `user.ErrNotFound`,
`postgres.UserStore`. The one accepted stutter is the package's main type
(`user.User`, like `time.Time`). Use short receiver names (`s`, `h`, `w`).

**Named types where mixing IDs would be a bug.** `type ID string` in each
domain package means `user.ID` and `site.ID` can't be swapped by accident.
Don't wrap every primitive.

---

## 4. Domain package: values, errors, service

`internal/user/user.go`: values, errors, and pure rules.

```go
// ID identifies a user. A named type keeps it from being mixed up with other IDs.
type ID string

// User is a registered user.
type User struct {
	ID    ID     `json:"id"`
	Email string `json:"email"`
	Name  string `json:"name"`
}

// RegisterInput is the data needed to register a user.
type RegisterInput struct {
	Email string `json:"email"`
	Name  string `json:"name"`
}

// ErrNotFound is returned when a user does not exist.
var ErrNotFound = errors.New("user not found")

// ErrEmailTaken is returned when the email is already registered.
var ErrEmailTaken = errors.New("email already registered")

// ValidationError describes invalid input for a single field.
type ValidationError struct {
	Field   string
	Message string
}

func (e *ValidationError) Error() string {
	return fmt.Sprintf("%s: %s", e.Field, e.Message)
}

// ValidateRegisterInput checks normalized input.
func ValidateRegisterInput(in RegisterInput) error {
	if _, err := mail.ParseAddress(in.Email); err != nil {
		return &ValidationError{Field: "email", Message: "must be a valid address"}
	}
	if in.Name == "" {
		return &ValidationError{Field: "name", Message: "is required"}
	}
	return nil
}
```

`internal/user/service.go`: the consumer-side interface and the service.

```go
// Store is what the service needs from storage. It is defined here, where it
// is consumed, and holds only the methods the service calls.
type Store interface {
	Insert(ctx context.Context, in RegisterInput) (User, error)
	Find(ctx context.Context, id ID) (User, error)
}

// Service orchestrates user operations.
type Service struct {
	store  Store
	logger *slog.Logger
}

// NewService returns a Service backed by store.
func NewService(store Store, logger *slog.Logger) *Service {
	return &Service{store: store, logger: logger}
}

// Register normalizes and validates input, then stores a new user.
func (s *Service) Register(ctx context.Context, in RegisterInput) (User, error) {
	in = NormalizeRegisterInput(in)

	if err := ValidateRegisterInput(in); err != nil {
		return User{}, err
	}

	u, err := s.store.Insert(ctx, in)
	if err != nil {
		return User{}, fmt.Errorf("insert user: %w", err)
	}

	s.logger.InfoContext(ctx, "user registered", "user_id", u.ID)
	return u, nil
}
```

Points to notice:

- **The service is a struct; the steps are functions.** `Service` exists to
  hold dependencies. Normalize and validate are free functions over values,
  so they can be tested without a service at all.
- **The interface lives with its consumer.** `user.Store` lists the two
  methods the service calls. `postgres.UserStore` satisfies it without
  importing anything to declare that. Add methods to the interface only when
  the consumer needs them.
- **The constructor is plain.** Its dependencies are explicit parameters. A
  component with several settings takes a `XConfig` struct. Functional
  options hide what a component needs, so leave them to library APIs.
- **The log line records an event, not an error.** Errors are returned, and
  the boundary that handles them logs them (section 9).

---

## 5. Storage

SQL is already a good query language, so write it directly. Use whatever the
project already uses (pgx, `database/sql`, or sqlc-generated code); for a new
Postgres project, use pgx. Keep each query as a named constant next to the
function that runs it, and translate driver errors into domain errors here,
at the boundary.

`internal/postgres/user.go`:

```go
// UserStore stores users. It satisfies user.Store.
type UserStore struct {
	db *pgxpool.Pool
}

const insertUserSQL = `
	INSERT INTO users (email, name)
	VALUES ($1, $2)
	RETURNING id, email, name`

// Insert stores a new user and returns it with its generated ID.
func (s *UserStore) Insert(ctx context.Context, in user.RegisterInput) (user.User, error) {
	var u user.User

	err := s.db.QueryRow(ctx, insertUserSQL, in.Email, in.Name).Scan(&u.ID, &u.Email, &u.Name)
	if pgErr, ok := errors.AsType[*pgconn.PgError](err); ok && pgErr.Code == "23505" {
		return user.User{}, user.ErrEmailTaken
	}
	if err != nil {
		return user.User{}, err
	}

	return u, nil
}

const findUserSQL = `
	SELECT id, email, name
	FROM users
	WHERE id = $1`

// Find returns the user with the given ID, or user.ErrNotFound.
func (s *UserStore) Find(ctx context.Context, id user.ID) (user.User, error) {
	var u user.User

	err := s.db.QueryRow(ctx, findUserSQL, id).Scan(&u.ID, &u.Email, &u.Name)
	if errors.Is(err, pgx.ErrNoRows) {
		return user.User{}, user.ErrNotFound
	}
	if err != nil {
		return user.User{}, fmt.Errorf("find user %s: %w", id, err)
	}

	return u, nil
}
```

With `database/sql`, the same code uses `*sql.DB`, `QueryRowContext`, and
`sql.ErrNoRows`. Before Go 1.26, write
`var pgErr *pgconn.PgError; errors.As(err, &pgErr)` instead of
`errors.AsType`.

Avoid ORMs, generic repositories, and query-builder frameworks. They add a
second query language and hide what actually reaches the database. sqlc is
different: it generates typed Go from plain SQL, so the SQL stays visible.

---

## 6. HTTP

Handlers translate between HTTP and application values: decode → call the
service → map the result or error → encode. Business decisions don't belong
here. On Go 1.22+, `ServeMux` handles methods and path parameters, so no
router dependency is needed.

`internal/httpapi/handler.go`:

```go
// NewHandler returns the API's http.Handler with all routes registered.
func NewHandler(users *user.Service, logger *slog.Logger) http.Handler {
	h := &Handler{users: users, logger: logger}

	mux := http.NewServeMux()
	mux.HandleFunc("POST /users", h.createUser)
	mux.HandleFunc("GET /users/{id}", h.getUser)
	return mux
}

func (h *Handler) createUser(w http.ResponseWriter, r *http.Request) {
	var in user.RegisterInput
	if err := json.NewDecoder(r.Body).Decode(&in); err != nil {
		writeError(w, http.StatusBadRequest, "invalid JSON body")
		return
	}

	u, err := h.users.Register(r.Context(), in)
	if err != nil {
		h.writeServiceError(w, r, err)
		return
	}

	writeJSON(w, http.StatusCreated, u)
}

func (h *Handler) getUser(w http.ResponseWriter, r *http.Request) {
	u, err := h.users.Get(r.Context(), user.ID(r.PathValue("id")))
	if err != nil {
		h.writeServiceError(w, r, err)
		return
	}

	writeJSON(w, http.StatusOK, u)
}

// writeServiceError maps domain errors to responses. It is the one place
// that logs unexpected errors; services return them without logging.
func (h *Handler) writeServiceError(w http.ResponseWriter, r *http.Request, err error) {
	if verr, ok := errors.AsType[*user.ValidationError](err); ok {
		writeError(w, http.StatusUnprocessableEntity, verr.Error())
		return
	}

	switch {
	case errors.Is(err, user.ErrNotFound):
		writeError(w, http.StatusNotFound, "user not found")
	case errors.Is(err, user.ErrEmailTaken):
		writeError(w, http.StatusConflict, "email already registered")
	default:
		h.logger.ErrorContext(r.Context(), "request failed",
			"method", r.Method, "path", r.URL.Path, "err", err)
		writeError(w, http.StatusInternalServerError, "internal error")
	}
}
```

Internal error details go to the log, not to the client. The routing table
in `NewHandler` is explicit registration: grep for a path and you'll find
its handler.

---

## 7. Composition root, lifecycle, and main

`internal/app` builds the graph from config, owns every long-lived resource,
and closes them all on shutdown. It plays the role of an Integrant system
map, written as ordinary code.

`internal/app/config.go`: config is a typed value, loaded and validated once.

```go
// Config is the whole application's configuration, loaded once at startup.
type Config struct {
	HTTPAddr        string
	DatabaseURL     string
	ShutdownTimeout time.Duration
}

// LoadConfig reads configuration from the environment and validates it.
func LoadConfig() (Config, error) {
	cfg := Config{
		HTTPAddr:        envOr("HTTP_ADDR", ":8080"),
		DatabaseURL:     os.Getenv("DATABASE_URL"),
		ShutdownTimeout: 10 * time.Second,
	}

	if cfg.DatabaseURL == "" {
		return Config{}, errors.New("DATABASE_URL is required")
	}

	return cfg, nil
}
```

Nothing below the composition root reads environment variables. Each
component gets its validated config, or the dependencies built from it.

`internal/app/app.go`:

```go
// App is the running system.
type App struct {
	cfg    Config
	logger *slog.Logger
	db     *pgxpool.Pool
	server *http.Server
}

// New builds the dependency graph. Read it top to bottom to see what depends
// on what.
func New(ctx context.Context, cfg Config, logger *slog.Logger) (*App, error) {
	db, err := pgxpool.New(ctx, cfg.DatabaseURL)
	if err != nil {
		return nil, fmt.Errorf("open database: %w", err)
	}

	users := user.NewService(postgres.NewUserStore(db), logger)
	handler := httpapi.NewHandler(users, logger)

	server := &http.Server{
		Addr:    cfg.HTTPAddr,
		Handler: handler,
	}

	return &App{cfg: cfg, logger: logger, db: db, server: server}, nil
}

// Run serves until ctx is cancelled or the server fails, then shuts
// everything down.
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

// close releases resources in reverse order of construction.
func (a *App) close(ctx context.Context) error {
	var errs []error

	if err := a.server.Shutdown(ctx); err != nil {
		errs = append(errs, fmt.Errorf("shutdown http: %w", err))
	}

	a.db.Close()

	return errors.Join(errs...)
}
```

Why `Run` looks like this:

- Both exit paths, a cancelled context and a failed server, go through
  `close`. That way the database pool gets closed even if `ListenAndServe`
  fails at startup.
- Shutdown runs under a fresh context with a timeout. The parent context is
  already cancelled at this point, and an unbounded `Shutdown` can hang
  forever on a stuck connection.
- The serve goroutine has an owner (`Run`), a way to stop (`Shutdown`), and
  an error destination (the buffered channel, so the goroutine never blocks
  after `Run` returns).

`cmd/shop/main.go`: `main` stays boring.

```go
func main() {
	if err := run(); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}

// run holds the program so that deferred cleanup runs before exit.
func run() error {
	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()

	logger := slog.New(slog.NewJSONHandler(os.Stdout, nil))

	cfg, err := app.LoadConfig()
	if err != nil {
		return fmt.Errorf("load config: %w", err)
	}

	a, err := app.New(ctx, cfg, logger)
	if err != nil {
		return err
	}

	return a.Run(ctx)
}
```

`log.Fatal` and `os.Exit` skip deferred calls. Keep them out of everything
except the last line of `main`.

---

## 8. Concurrency, workers, and stateful types

Code is synchronous by default. Add goroutines where they solve a concrete
problem: serving requests, independent I/O, background workers, bounded
pipelines, or fan-out/fan-in that clearly pays off. Don't spread channels
through domain logic. A channel is a concurrency primitive, not a way to
structure an application.

Every long-lived goroutine should have a clear answer to these: who starts
it, who owns it, how it's cancelled, who waits for it, and where its errors
go.

`internal/analytics/worker.go`:

```go
// Worker writes events from input to the store until ctx is cancelled or
// input is closed.
type Worker struct {
	store EventStore
	input <-chan Event
}

// Run blocks until ctx is done, input is closed, or a store write fails.
func (w *Worker) Run(ctx context.Context) error {
	for {
		select {
		case <-ctx.Done():
			return ctx.Err()

		case e, ok := <-w.input:
			if !ok {
				return nil
			}
			if err := w.store.InsertEvent(ctx, e); err != nil {
				return fmt.Errorf("insert event %s: %w", e.Name, err)
			}
		}
	}
}
```

Without the `ok` check, a closed channel delivers zero values forever and the
worker spins. The composition root starts workers and waits for them: on Go
1.25+ with `wg.Go(func() { errs <- w.Run(ctx) })`, and before that with
`wg.Add`/`Done`. Don't add a distributed queue until the workload needs one.

**Shared mutable state lives in a type that owns it**, and that type uses
pointer receivers. A value receiver would copy the mutex, and `go vet`'s
copylocks check flags that.

```go
// Counter counts events by name. It owns its state, so it takes pointer
// receivers: a value receiver would copy the mutex.
type Counter struct {
	mu     sync.Mutex
	counts map[string]int
}

func (c *Counter) Add(name string) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.counts[name]++
}
```

No package-level `var db`, `var config`, or `var cache`. Build shared
components in the composition root and pass them down.

---

## 9. Errors and logging

- **Wrap where it adds context** that the next reader needs: which operation,
  which ID. `fmt.Errorf("find user %s: %w", id, err)`. Don't wrap every line.
  A chain like `serve http: insert user: find user 42: connection refused`
  should read as one sentence. That's why messages are lowercase and don't
  start with "failed to" or "error".
- **Sentinel and typed errors exist for callers that branch on them.**
  `ErrNotFound` and `ErrEmailTaken` drive HTTP status codes, and
  `*ValidationError` carries a field name. If no caller branches on an error,
  a plain wrapped error is enough.
- **Translate at boundaries.** Storage turns `pgx.ErrNoRows` and unique
  violations into domain errors, and HTTP turns domain errors into status
  codes. Domain code never sees driver errors.
- **Log or return, never both.** If every layer logs and then returns, one
  failure shows up four times in the logs. The code that finally handles an
  error logs it once: the HTTP error mapper, a worker's supervisor, `main`.
  Ordinary events (a user registered, the server listening) can be logged
  wherever they happen.
- **Use `slog` with structured key-value pairs** and the `...Context`
  variants so request metadata comes along. Pass the logger as a
  dependency; don't use a global logger.
- **Panic only for broken invariants** (a programmer error), never for
  input, I/O, or missing rows.
- **`context.Context`** carries cancellation, deadlines, and request-scoped
  metadata such as trace IDs. Dependencies always go in parameters or
  struct fields, never in `ctx.Value`.

---

## 10. Tests

Test behavior through public functions and service methods. testify is
allowed: use `require` when a failed check should stop the test and `assert`
for everything else. If the project uses only the standard library, follow
it.

A hand-written fake for the consumer-side interface takes a few lines and
doesn't tie tests to call order the way generated mocks do:

```go
// fakeStore is an in-memory user.Store for service tests.
type fakeStore struct {
	byID    map[user.ID]user.User
	byEmail map[string]user.ID
}

func newFakeStore() *fakeStore {
	return &fakeStore{byID: map[user.ID]user.User{}, byEmail: map[string]user.ID{}}
}

func (f *fakeStore) Insert(_ context.Context, in user.RegisterInput) (user.User, error) {
	if _, ok := f.byEmail[in.Email]; ok {
		return user.User{}, user.ErrEmailTaken
	}
	u := user.User{ID: user.ID(fmt.Sprint(len(f.byID) + 1)), Email: in.Email, Name: in.Name}
	f.byID[u.ID] = u
	f.byEmail[u.Email] = u.ID
	return u, nil
}

func newTestService(t *testing.T) *user.Service {
	t.Helper()
	return user.NewService(newFakeStore(), slog.New(slog.DiscardHandler))
}

func TestRegisterRejectsDuplicateEmail(t *testing.T) {
	svc := newTestService(t)
	in := user.RegisterInput{Email: "ann@example.com", Name: "Ann"}

	_, err := svc.Register(t.Context(), in)
	require.NoError(t, err)

	_, err = svc.Register(t.Context(), in)
	assert.ErrorIs(t, err, user.ErrEmailTaken)
}
```

Use a table when the cases share a shape:

```go
func TestValidateRegisterInput(t *testing.T) {
	tests := []struct {
		name  string
		in    user.RegisterInput
		field string
	}{
		{"bad email", user.RegisterInput{Email: "nope", Name: "Ann"}, "email"},
		{"missing name", user.RegisterInput{Email: "ann@example.com"}, "name"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			verr, ok := errors.AsType[*user.ValidationError](user.ValidateRegisterInput(tt.in))
			require.True(t, ok)
			assert.Equal(t, tt.field, verr.Field)
		})
	}
}
```

Write a plain test when a table would obscure what's being checked.

- `t.Context()` needs Go 1.24+, `slog.DiscardHandler` 1.24+, and
  `errors.AsType` 1.26+. On older versions use `context.Background()`,
  `slog.New(slog.NewTextHandler(io.Discard, nil))`, and `errors.As`.
- Test storage code against a real database through the project's existing
  setup (docker compose or testcontainers). A SQL mock only checks that you
  typed the query you typed.
- Tests build everything they need. With no package state there's nothing
  to reset, and `t.Parallel()` is safe.
- Run tests with `-race`.

---

## 11. Abstractions that must earn their place

- **Interfaces.** Write concrete code first. Introduce an interface where it's
  consumed, once there's a second implementation or a fake, and keep only the
  methods that consumer uses. A generic `Repository[T]` with eight methods is
  architecture-by-interface. Three concrete functions are easier to read.
- **Generics.** A small generic helper that's used in several places is fine.
  Type-level frameworks (`AbstractRepository[Entity, ID, Filter]`) aren't.
- **FP libraries.** A loop with `append` is idiomatic Go's map/filter:

  ```go
  emails := make([]string, 0, len(users))
  for _, u := range users {
  	if u.Active {
  		emails = append(emails, u.Email)
  	}
  }
  ```

  The FP ideas worth keeping are clear transformations, minimal hidden
  state, and explicit inputs and outputs, not `Map`/`Filter`/`Reduce` syntax.
  For sorting, searching, and set-like operations, use `slices` and `maps`.
- **Frameworks and internal platforms.** Don't build an internal framework
  just so every service looks the same. Extract a library only after several
  real applications show the same pattern.
- **Code generation.** It's worth it when it removes runtime reflection and
  adds type safety (sqlc). It isn't worth it as a layer to avoid typing.
- **Embedding.** Embed for composition when it's genuinely useful. Don't
  build inheritance-like chains of embedded structs.

---

## 12. Deployment shape

Where it fits, ship one binary. A small web app can embed its frontend:

```go
//go:embed dist
var frontend embed.FS
```

Keep external files for real data and configuration. Don't force a single
binary when it makes the architecture worse, and don't split into
microservices before the workload needs it.

The resulting codebase should be boring to operate, easy to navigate,
change, and debug, easy for agents to modify, and easy for humans to review.
It should also keep Go's strengths: fast builds, static types, native
binaries, and simple deployment.
