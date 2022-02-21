-- Your SQL goes here
CREATE TABLE payout_splits (
    id TEXT PRIMARY KEY NOT NULL,
    created_at TIMESTAMP NOT NULL,
    store_id TEXT,
    store_percentage DOUBLE PRECISION,
    platform_percentage DOUBLE PRECISION,
    affiliate_id TEXT,
    affiliate_percentage DOUBLE PRECISION
);
