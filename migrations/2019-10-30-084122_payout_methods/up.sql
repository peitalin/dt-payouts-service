-- Your SQL goes here

CREATE TABLE payout_methods (
    id TEXT PRIMARY KEY NOT NULL,
    payee_id TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP,
    payout_type TEXT,
    payout_email TEXT,
    payout_processor TEXT,
    payout_processor_id TEXT
);
