-- Your SQL goes here
CREATE TABLE payment_method_addresses (
    -- stripe/paypal payment_method id
    payment_method_id TEXT PRIMARY KEY,
    line1 TEXT,
    line2 TEXT,
    city TEXT,
    state TEXT,
    postal_code TEXT,
    country TEXT,
    town TEXT
);

