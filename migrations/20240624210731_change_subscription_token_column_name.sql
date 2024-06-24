-- Add migration script here
ALTER TABLE subscription_tokens RENAME COLUMN subscription_id to subscriber_id;