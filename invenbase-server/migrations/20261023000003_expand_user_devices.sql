DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'user_devices' AND column_name = 'fcm_token'
    ) AND NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'user_devices' AND column_name = 'push_token'
    ) THEN
        ALTER TABLE user_devices RENAME COLUMN fcm_token TO push_token;
    END IF;
END $$;

ALTER TABLE user_devices ADD COLUMN IF NOT EXISTS device_id TEXT;
ALTER TABLE user_devices ADD COLUMN IF NOT EXISTS provider TEXT NOT NULL DEFAULT 'fcm';
ALTER TABLE user_devices ADD COLUMN IF NOT EXISTS active BOOLEAN NOT NULL DEFAULT TRUE;
ALTER TABLE user_devices ADD COLUMN IF NOT EXISTS last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

UPDATE user_devices
SET device_id = 'legacy:' || id::text
WHERE device_id IS NULL OR BTRIM(device_id) = '';

ALTER TABLE user_devices ALTER COLUMN device_id SET NOT NULL;

DROP INDEX IF EXISTS user_devices_user_token_idx;

-- Old clients could associate the same FCM token with different accounts.
-- Keep the most recently seen row before enforcing provider/token ownership.
WITH ranked_devices AS (
    SELECT id,
           ROW_NUMBER() OVER (
               PARTITION BY provider, push_token
               ORDER BY updated_at DESC NULLS LAST, created_at DESC NULLS LAST, id DESC
           ) AS duplicate_rank
    FROM user_devices
)
DELETE FROM user_devices
WHERE id IN (
    SELECT id FROM ranked_devices WHERE duplicate_rank > 1
);

CREATE UNIQUE INDEX IF NOT EXISTS user_devices_user_provider_device_idx
    ON user_devices(user_id, provider, device_id);
CREATE UNIQUE INDEX IF NOT EXISTS user_devices_provider_token_idx
    ON user_devices(provider, push_token);
CREATE INDEX IF NOT EXISTS idx_user_devices_active_user
    ON user_devices(user_id, active);

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'user_devices_provider_check'
          AND conrelid = 'user_devices'::regclass
    ) THEN
        ALTER TABLE user_devices
            ADD CONSTRAINT user_devices_provider_check
            CHECK (provider IN ('fcm', 'rustore', 'huawei')) NOT VALID;
    END IF;
END $$;
