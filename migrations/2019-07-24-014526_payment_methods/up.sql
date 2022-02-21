-- Your SQL goes here
CREATE TABLE payment_methods (
    -- stripe/paypal payment_method id
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP,
    -- customer id
    customer_id TEXT,
    -- paypal | stripe
    payment_processor TEXT,
    -- card, paypal, token, bank
    payment_method_types TEXT[],
    -- last 4 digits of card
    last4 TEXT,
    -- expiry month
    exp_month INT,
    -- expiry year
    exp_year INT,
    -- name associated with card
    name TEXT,
    -- email associated with card
    email TEXT,
    -- other details of payment method
    details TEXT
);


CREATE OR REPLACE FUNCTION trigger_set_timestamp()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_timestamp
BEFORE UPDATE ON payment_methods
FOR EACH ROW
EXECUTE PROCEDURE trigger_set_timestamp();