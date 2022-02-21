-- Your SQL goes here

ALTER TABLE payout_splits
DROP COLUMN store_id,
DROP COLUMN store_percentage,
DROP COLUMN platform_percentage,
DROP COLUMN affiliate_id,
DROP COLUMN affiliate_percentage,
ADD COLUMN store_or_user_id TEXT NOT NULL,
ADD COLUMN deal_type TEXT NOT NULL,
ADD COLUMN expires_at TIMESTAMP,
ADD COLUMN rate DOUBLE PRECISION NOT NULL,
ADD COLUMN referrer_id TEXT;
