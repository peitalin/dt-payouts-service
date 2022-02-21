-- Your SQL goes here
CREATE TABLE refunds (
    id TEXT PRIMARY KEY,
    transaction_id TEXT NOT NULL,
    order_id TEXT NOT NULL,
    order_item_ids TEXT[], -- refunded items
    created_at TIMESTAMP DEFAULT NOW() NOT NULL,
    reason TEXT,
    reason_details TEXT
);
