-- Add migration script here
-- wrap in in transaction for rollback incase of failure
BEGIN;
    UPDATE subscriptions
        set status = 'confirmed'
        WHERE status IS NULL;
    ALTER TABLE subscriptions ALTER COLUMN status SET NOT NULL;
COMMIT;
