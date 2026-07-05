CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(100) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    email VARCHAR(255),
    full_name VARCHAR(255),
    role VARCHAR(20) NOT NULL DEFAULT 'user',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS squads (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    location VARCHAR(255),
    responsible_user_id UUID REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS equipment_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    squad_id UUID REFERENCES squads(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS equipment (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    category_id UUID REFERENCES equipment_categories(id),
    squad_id UUID REFERENCES squads(id),
    quantity INTEGER NOT NULL DEFAULT 1,
    available_quantity INTEGER NOT NULL DEFAULT 1,
    is_unique BOOLEAN NOT NULL DEFAULT false,
    location VARCHAR(255),
    qr_code VARCHAR(255) UNIQUE,
    responsible_user_id UUID REFERENCES users(id),
    status VARCHAR(50) DEFAULT 'available',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS equipment_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS equipment_group_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES equipment_groups(id) ON DELETE CASCADE,
    equipment_id UUID NOT NULL REFERENCES equipment(id) ON DELETE CASCADE,
    quantity INTEGER NOT NULL DEFAULT 1,
    UNIQUE(group_id, equipment_id)
);

CREATE TABLE IF NOT EXISTS bookings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    equipment_id UUID REFERENCES equipment(id),
    group_id UUID REFERENCES equipment_groups(id),
    quantity INTEGER NOT NULL DEFAULT 1,
    start_date TIMESTAMP WITH TIME ZONE NOT NULL,
    end_date TIMESTAMP WITH TIME ZONE NOT NULL,
    purpose TEXT,
    status VARCHAR(50) DEFAULT 'pending',
    permission_type VARCHAR(50) DEFAULT 'internal',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    booking_id UUID NOT NULL REFERENCES bookings(id) ON DELETE CASCADE,
    equipment_id UUID REFERENCES equipment(id),
    permission_type VARCHAR(50) NOT NULL,
    issued_by UUID REFERENCES users(id),
    issued_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP WITH TIME ZONE,
    status VARCHAR(50) DEFAULT 'active'
);

CREATE TABLE IF NOT EXISTS activity_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    action VARCHAR(100) NOT NULL,
    entity_type VARCHAR(50) NOT NULL,
    entity_id UUID,
    details JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    notification_type VARCHAR(50) NOT NULL,
    is_read BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS equipment_movements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    equipment_id UUID NOT NULL REFERENCES equipment(id) ON DELETE CASCADE,
    from_squad_id UUID,
    to_squad_id UUID,
    from_location VARCHAR(255),
    to_location VARCHAR(255),
    moved_by UUID REFERENCES users(id),
    comment TEXT,
    moved_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS user_devices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    fcm_token TEXT NOT NULL,
    platform TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS support_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    subject VARCHAR(500) NOT NULL,
    message TEXT NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'open',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    admin_comment TEXT
);

CREATE TABLE IF NOT EXISTS support_request_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    support_request_id UUID NOT NULL REFERENCES support_requests(id) ON DELETE CASCADE,
    author_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    is_staff BOOLEAN NOT NULL DEFAULT FALSE,
    message TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE equipment ADD COLUMN IF NOT EXISTS squad_id UUID REFERENCES squads(id);
ALTER TABLE equipment_categories ADD COLUMN IF NOT EXISTS squad_id UUID REFERENCES squads(id);
ALTER TABLE equipment ADD COLUMN IF NOT EXISTS is_unique BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE support_requests ADD COLUMN IF NOT EXISTS admin_comment TEXT;

CREATE INDEX IF NOT EXISTS idx_equipment_category ON equipment(category_id);
CREATE INDEX IF NOT EXISTS idx_equipment_qr ON equipment(qr_code);
CREATE INDEX IF NOT EXISTS idx_bookings_user ON bookings(user_id);
CREATE INDEX IF NOT EXISTS idx_bookings_equipment ON bookings(equipment_id);
CREATE INDEX IF NOT EXISTS idx_bookings_status ON bookings(status);
CREATE INDEX IF NOT EXISTS idx_activity_logs_user ON activity_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_activity_logs_created_at ON activity_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_notifications_user ON notifications(user_id);
CREATE UNIQUE INDEX IF NOT EXISTS user_devices_user_token_idx ON user_devices(user_id, fcm_token);
CREATE INDEX IF NOT EXISTS idx_user_devices_user ON user_devices(user_id);
CREATE INDEX IF NOT EXISTS idx_squads_responsible ON squads(responsible_user_id);
CREATE INDEX IF NOT EXISTS idx_equipment_squad ON equipment(squad_id);
CREATE INDEX IF NOT EXISTS idx_equipment_categories_squad ON equipment_categories(squad_id);
CREATE INDEX IF NOT EXISTS idx_equipment_movements_equipment ON equipment_movements(equipment_id);
CREATE INDEX IF NOT EXISTS idx_equipment_movements_moved_at ON equipment_movements(moved_at);
CREATE INDEX IF NOT EXISTS idx_support_requests_user ON support_requests(user_id);
CREATE INDEX IF NOT EXISTS idx_support_requests_status ON support_requests(status);
CREATE INDEX IF NOT EXISTS idx_support_request_messages_request ON support_request_messages(support_request_id);

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'users_role_check' AND conrelid = 'users'::regclass) THEN
        ALTER TABLE users ADD CONSTRAINT users_role_check CHECK (role IN ('admin', 'responsible', 'user')) NOT VALID;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'equipment_quantity_positive_check' AND conrelid = 'equipment'::regclass) THEN
        ALTER TABLE equipment ADD CONSTRAINT equipment_quantity_positive_check CHECK (quantity > 0) NOT VALID;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'equipment_available_quantity_non_negative_check' AND conrelid = 'equipment'::regclass) THEN
        ALTER TABLE equipment ADD CONSTRAINT equipment_available_quantity_non_negative_check CHECK (available_quantity >= 0) NOT VALID;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'equipment_available_quantity_lte_quantity_check' AND conrelid = 'equipment'::regclass) THEN
        ALTER TABLE equipment ADD CONSTRAINT equipment_available_quantity_lte_quantity_check CHECK (available_quantity <= quantity) NOT VALID;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'equipment_status_check' AND conrelid = 'equipment'::regclass) THEN
        ALTER TABLE equipment ADD CONSTRAINT equipment_status_check CHECK (status IN ('available', 'maintenance', 'unavailable')) NOT VALID;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'equipment_group_items_quantity_positive_check' AND conrelid = 'equipment_group_items'::regclass) THEN
        ALTER TABLE equipment_group_items ADD CONSTRAINT equipment_group_items_quantity_positive_check CHECK (quantity > 0) NOT VALID;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'bookings_quantity_positive_check' AND conrelid = 'bookings'::regclass) THEN
        ALTER TABLE bookings ADD CONSTRAINT bookings_quantity_positive_check CHECK (quantity > 0) NOT VALID;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'bookings_date_order_check' AND conrelid = 'bookings'::regclass) THEN
        ALTER TABLE bookings ADD CONSTRAINT bookings_date_order_check CHECK (end_date > start_date) NOT VALID;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'bookings_status_check' AND conrelid = 'bookings'::regclass) THEN
        ALTER TABLE bookings ADD CONSTRAINT bookings_status_check CHECK (status IN ('pending', 'approved', 'rejected', 'cancelled', 'expired', 'awaiting_return', 'returned', 'completed')) NOT VALID;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'bookings_permission_type_check' AND conrelid = 'bookings'::regclass) THEN
        ALTER TABLE bookings ADD CONSTRAINT bookings_permission_type_check CHECK (permission_type IN ('internal', 'external')) NOT VALID;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'bookings_target_check' AND conrelid = 'bookings'::regclass) THEN
        ALTER TABLE bookings ADD CONSTRAINT bookings_target_check CHECK (equipment_id IS NOT NULL OR group_id IS NOT NULL) NOT VALID;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'permissions_status_check' AND conrelid = 'permissions'::regclass) THEN
        ALTER TABLE permissions ADD CONSTRAINT permissions_status_check CHECK (status IN ('active', 'revoked')) NOT VALID;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'permissions_permission_type_check' AND conrelid = 'permissions'::regclass) THEN
        ALTER TABLE permissions ADD CONSTRAINT permissions_permission_type_check CHECK (permission_type IN ('internal', 'external')) NOT VALID;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'support_requests_status_check' AND conrelid = 'support_requests'::regclass) THEN
        ALTER TABLE support_requests ADD CONSTRAINT support_requests_status_check CHECK (status IN ('open', 'in_progress', 'answered', 'closed')) NOT VALID;
    END IF;
END $$;
