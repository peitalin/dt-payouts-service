-- This file should undo anything in `up.sql`

ALTER TABLE payout_splits
ADD COLUMN store_id TEXT,
ADD COLUMN store_percentage DOUBLE PRECISION,
ADD COLUMN platform_percentage DOUBLE PRECISION,
ADD COLUMN affiliate_id TEXT,
ADD COLUMN affiliate_percentage DOUBLE PRECISION,

DROP COLUMN store_or_user_id,
DROP COLUMN deal_type,
DROP COLUMN expires_at,
DROP COLUMN rate,
DROP COLUMN referrer_id;