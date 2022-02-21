-- Your SQL goes here
CREATE TABLE transactions (
    -- payment intent id
    id TEXT PRIMARY KEY NOT NULL,
    subtotal INT NOT NULL,
    taxes INT NOT NULL,
    payment_processing_fee INT NOT NULL,
    -- seller_payment INT NOT NULL,
    -- platform_fee INT NOT NULL,
    -- affiliate_fee INT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW() NOT NULL,
    currency TEXT,
    -- stripe charge id
    charge_id TEXT,
    customer_id TEXT,
    order_id TEXT,
    -- paypal | stripe
    payment_processor TEXT,
    -- stripe/paypal payment_method id
    payment_method_id TEXT,
    -- stripe payment_intent id
    payment_intent_id TEXT,
    refund_id TEXT,
    details TEXT
);

