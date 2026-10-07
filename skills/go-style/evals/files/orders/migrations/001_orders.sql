CREATE TABLE orders (
    id          BIGSERIAL PRIMARY KEY,
    customer_id TEXT        NOT NULL,
    status      TEXT        NOT NULL DEFAULT 'pending',
    total_cents BIGINT      NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
