-- Add migration script here
ALTER TABLE api_keys
ADD COLUMN quota_limit INTEGER DEFAULT NULL,
ADD COLUMN rate_limit_per_minute INTEGER DEFAULT 60;

UPDATE api_keys SET quota_limit = 1000 WHERE quota_limit IS NULL;