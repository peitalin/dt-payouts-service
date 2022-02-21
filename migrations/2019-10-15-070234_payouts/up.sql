-- Your SQL goes here
CREATE TABLE payouts (
    id TEXT PRIMARY KEY NOT NULL,
    payee_id TEXT NOT NULL,
    payee_type TEXT NOT NULL,
    amount INT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    start_period TIMESTAMP,
    end_period TIMESTAMP,
    payout_date TIMESTAMP,
    payout_status TEXT NOT NULL,
    payout_email TEXT NOT NULL,
    currency TEXT NOT NULL,
    payout_item_ids TEXT[] NOT NULL,
    approved_by_ids TEXT[] NOT NULL,
    payout_batch_id TEXT,
    details TEXT
);

