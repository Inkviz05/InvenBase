CREATE TABLE notification_outbox (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    body TEXT NOT NULL,
    data JSONB NOT NULL DEFAULT '{}'::jsonb,
    exclude_tokens JSONB NOT NULL DEFAULT '[]'::jsonb,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 5,
    next_attempt_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    locked_at TIMESTAMPTZ,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMPTZ,
    CONSTRAINT notification_outbox_status_check
        CHECK (status IN ('pending', 'processing', 'delivered', 'failed')),
    CONSTRAINT notification_outbox_attempts_check CHECK (attempts >= 0),
    CONSTRAINT notification_outbox_max_attempts_check CHECK (max_attempts > 0),
    CONSTRAINT notification_outbox_data_check CHECK (jsonb_typeof(data) = 'object'),
    CONSTRAINT notification_outbox_exclude_tokens_check
        CHECK (jsonb_typeof(exclude_tokens) = 'array')
);

CREATE INDEX idx_notification_outbox_pending
    ON notification_outbox (next_attempt_at, created_at)
    WHERE status IN ('pending', 'processing');

CREATE INDEX idx_notification_outbox_user
    ON notification_outbox (user_id, created_at DESC);
