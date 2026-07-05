-- Run before adding VALIDATE CONSTRAINT migrations on an existing database.
-- The query returns rows that violate the same rules as current NOT VALID checks.

SELECT
    'users' AS table_name,
    'users_role_check' AS check_name,
    id::text AS row_id,
    jsonb_build_object('role', role) AS details
FROM users
WHERE NOT (role IN ('admin', 'responsible', 'user'))

UNION ALL
SELECT
    'equipment',
    'equipment_quantity_positive_check',
    id::text,
    jsonb_build_object('quantity', quantity)
FROM equipment
WHERE NOT (quantity > 0)

UNION ALL
SELECT
    'equipment',
    'equipment_available_quantity_non_negative_check',
    id::text,
    jsonb_build_object('available_quantity', available_quantity)
FROM equipment
WHERE NOT (available_quantity >= 0)

UNION ALL
SELECT
    'equipment',
    'equipment_available_quantity_lte_quantity_check',
    id::text,
    jsonb_build_object('quantity', quantity, 'available_quantity', available_quantity)
FROM equipment
WHERE NOT (available_quantity <= quantity)

UNION ALL
SELECT
    'equipment',
    'equipment_status_check',
    id::text,
    jsonb_build_object('status', status)
FROM equipment
WHERE NOT (status IN ('available', 'maintenance', 'unavailable'))

UNION ALL
SELECT
    'equipment_group_items',
    'equipment_group_items_quantity_positive_check',
    id::text,
    jsonb_build_object('quantity', quantity)
FROM equipment_group_items
WHERE NOT (quantity > 0)

UNION ALL
SELECT
    'bookings',
    'bookings_quantity_positive_check',
    id::text,
    jsonb_build_object('quantity', quantity)
FROM bookings
WHERE NOT (quantity > 0)

UNION ALL
SELECT
    'bookings',
    'bookings_date_order_check',
    id::text,
    jsonb_build_object('start_date', start_date, 'end_date', end_date)
FROM bookings
WHERE NOT (end_date > start_date)

UNION ALL
SELECT
    'bookings',
    'bookings_status_check',
    id::text,
    jsonb_build_object('status', status)
FROM bookings
WHERE NOT (status IN ('pending', 'approved', 'rejected', 'cancelled', 'expired', 'awaiting_return', 'returned', 'completed'))

UNION ALL
SELECT
    'bookings',
    'bookings_permission_type_check',
    id::text,
    jsonb_build_object('permission_type', permission_type)
FROM bookings
WHERE NOT (permission_type IN ('internal', 'external'))

UNION ALL
SELECT
    'bookings',
    'bookings_target_check',
    id::text,
    jsonb_build_object('equipment_id', equipment_id, 'group_id', group_id)
FROM bookings
WHERE NOT (equipment_id IS NOT NULL OR group_id IS NOT NULL)

UNION ALL
SELECT
    'permissions',
    'permissions_status_check',
    id::text,
    jsonb_build_object('status', status)
FROM permissions
WHERE NOT (status IN ('active', 'revoked'))

UNION ALL
SELECT
    'permissions',
    'permissions_permission_type_check',
    id::text,
    jsonb_build_object('permission_type', permission_type)
FROM permissions
WHERE NOT (permission_type IN ('internal', 'external'))

UNION ALL
SELECT
    'support_requests',
    'support_requests_status_check',
    id::text,
    jsonb_build_object('status', status)
FROM support_requests
WHERE NOT (status IN ('open', 'in_progress', 'answered', 'closed'))

ORDER BY table_name, check_name, row_id;
