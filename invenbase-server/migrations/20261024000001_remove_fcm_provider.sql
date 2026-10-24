-- FCM is no longer supported. Remove its device tokens and restrict new rows
-- to the remaining Android push transports.
DELETE FROM user_devices WHERE provider = 'fcm';

ALTER TABLE user_devices DROP CONSTRAINT IF EXISTS user_devices_provider_check;
ALTER TABLE user_devices ALTER COLUMN provider DROP DEFAULT;
ALTER TABLE user_devices
    ADD CONSTRAINT user_devices_provider_check
    CHECK (provider IN ('rustore', 'huawei'));
