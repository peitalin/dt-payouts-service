-- Your SQL goes here
CREATE TABLE payout_items (
    id TEXT PRIMARY KEY NOT NULL,
    payee_id TEXT NOT NULL,
    payee_type TEXT NOT NULL,
    amount INT NOT NULL,
    payment_processing_fee INT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    payout_status TEXT NOT NULL,
    currency TEXT NOT NULL,
    order_item_id TEXT NOT NULL,
    txn_id TEXT NOT NULL,
    payout_id TEXT
);

