-- This file should undo anything in `up.sql`
ALTER TABLE payouts
DROP COLUMN paid_to_payment_method_id;