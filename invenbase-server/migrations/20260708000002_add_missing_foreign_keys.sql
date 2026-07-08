DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'squads_responsible_user_id_fkey' AND conrelid = 'squads'::regclass) THEN
        ALTER TABLE squads ADD CONSTRAINT squads_responsible_user_id_fkey FOREIGN KEY (responsible_user_id) REFERENCES users(id) NOT VALID;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'equipment_categories_squad_id_fkey' AND conrelid = 'equipment_categories'::regclass) THEN
        ALTER TABLE equipment_categories ADD CONSTRAINT equipment_categories_squad_id_fkey FOREIGN KEY (squad_id) REFERENCES squads(id) NOT VALID;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'equipment_category_id_fkey' AND conrelid = 'equipment'::regclass) THEN
        ALTER TABLE equipment ADD CONSTRAINT equipment_category_id_fkey FOREIGN KEY (category_id) REFERENCES equipment_categories(id) NOT VALID;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'equipment_squad_id_fkey' AND conrelid = 'equipment'::regclass) THEN
        ALTER TABLE equipment ADD CONSTRAINT equipment_squad_id_fkey FOREIGN KEY (squad_id) REFERENCES squads(id) NOT VALID;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'equipment_responsible_user_id_fkey' AND conrelid = 'equipment'::regclass) THEN
        ALTER TABLE equipment ADD CONSTRAINT equipment_responsible_user_id_fkey FOREIGN KEY (responsible_user_id) REFERENCES users(id) NOT VALID;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'equipment_group_items_group_id_fkey' AND conrelid = 'equipment_group_items'::regclass) THEN
        ALTER TABLE equipment_group_items ADD CONSTRAINT equipment_group_items_group_id_fkey FOREIGN KEY (group_id) REFERENCES equipment_groups(id) ON DELETE CASCADE NOT VALID;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'equipment_group_items_equipment_id_fkey' AND conrelid = 'equipment_group_items'::regclass) THEN
        ALTER TABLE equipment_group_items ADD CONSTRAINT equipment_group_items_equipment_id_fkey FOREIGN KEY (equipment_id) REFERENCES equipment(id) ON DELETE CASCADE NOT VALID;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'bookings_user_id_fkey' AND conrelid = 'bookings'::regclass) THEN
        ALTER TABLE bookings ADD CONSTRAINT bookings_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(id) NOT VALID;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'bookings_equipment_id_fkey' AND conrelid = 'bookings'::regclass) THEN
        ALTER TABLE bookings ADD CONSTRAINT bookings_equipment_id_fkey FOREIGN KEY (equipment_id) REFERENCES equipment(id) NOT VALID;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'bookings_group_id_fkey' AND conrelid = 'bookings'::regclass) THEN
        ALTER TABLE bookings ADD CONSTRAINT bookings_group_id_fkey FOREIGN KEY (group_id) REFERENCES equipment_groups(id) NOT VALID;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'permissions_booking_id_fkey' AND conrelid = 'permissions'::regclass) THEN
        ALTER TABLE permissions ADD CONSTRAINT permissions_booking_id_fkey FOREIGN KEY (booking_id) REFERENCES bookings(id) ON DELETE CASCADE NOT VALID;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'permissions_equipment_id_fkey' AND conrelid = 'permissions'::regclass) THEN
        ALTER TABLE permissions ADD CONSTRAINT permissions_equipment_id_fkey FOREIGN KEY (equipment_id) REFERENCES equipment(id) NOT VALID;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'permissions_issued_by_fkey' AND conrelid = 'permissions'::regclass) THEN
        ALTER TABLE permissions ADD CONSTRAINT permissions_issued_by_fkey FOREIGN KEY (issued_by) REFERENCES users(id) NOT VALID;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'activity_logs_user_id_fkey' AND conrelid = 'activity_logs'::regclass) THEN
        ALTER TABLE activity_logs ADD CONSTRAINT activity_logs_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(id) NOT VALID;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'notifications_user_id_fkey' AND conrelid = 'notifications'::regclass) THEN
        ALTER TABLE notifications ADD CONSTRAINT notifications_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(id) NOT VALID;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'equipment_movements_equipment_id_fkey' AND conrelid = 'equipment_movements'::regclass) THEN
        ALTER TABLE equipment_movements ADD CONSTRAINT equipment_movements_equipment_id_fkey FOREIGN KEY (equipment_id) REFERENCES equipment(id) ON DELETE CASCADE NOT VALID;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'equipment_movements_moved_by_fkey' AND conrelid = 'equipment_movements'::regclass) THEN
        ALTER TABLE equipment_movements ADD CONSTRAINT equipment_movements_moved_by_fkey FOREIGN KEY (moved_by) REFERENCES users(id) NOT VALID;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'user_devices_user_id_fkey' AND conrelid = 'user_devices'::regclass) THEN
        ALTER TABLE user_devices ADD CONSTRAINT user_devices_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE NOT VALID;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'support_requests_user_id_fkey' AND conrelid = 'support_requests'::regclass) THEN
        ALTER TABLE support_requests ADD CONSTRAINT support_requests_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE NOT VALID;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'support_request_messages_support_request_id_fkey' AND conrelid = 'support_request_messages'::regclass) THEN
        ALTER TABLE support_request_messages ADD CONSTRAINT support_request_messages_support_request_id_fkey FOREIGN KEY (support_request_id) REFERENCES support_requests(id) ON DELETE CASCADE NOT VALID;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'support_request_messages_author_user_id_fkey' AND conrelid = 'support_request_messages'::regclass) THEN
        ALTER TABLE support_request_messages ADD CONSTRAINT support_request_messages_author_user_id_fkey FOREIGN KEY (author_user_id) REFERENCES users(id) ON DELETE SET NULL NOT VALID;
    END IF;
END $$;
