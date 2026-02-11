use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;

pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(5))
            .connect(database_url)
            .await?;

        Ok(Database { pool })
    }

    pub async fn init(&self) -> Result<(), sqlx::Error> {
        // Создание таблицы пользователей
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                username VARCHAR(100) UNIQUE NOT NULL,
                password_hash VARCHAR(255) NOT NULL,
                email VARCHAR(255),
                full_name VARCHAR(255),
                role VARCHAR(20) NOT NULL DEFAULT 'user',
                created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Создание таблицы категорий оборудования
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS equipment_categories (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                name VARCHAR(255) NOT NULL,
                description TEXT,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Создание таблицы оборудования
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS equipment (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                name VARCHAR(255) NOT NULL,
                description TEXT,
                category_id UUID REFERENCES equipment_categories(id),
                quantity INTEGER NOT NULL DEFAULT 1,
                available_quantity INTEGER NOT NULL DEFAULT 1,
                is_unique BOOLEAN NOT NULL DEFAULT false,
                location VARCHAR(255),
                qr_code VARCHAR(255) UNIQUE,
                responsible_user_id UUID REFERENCES users(id),
                status VARCHAR(50) DEFAULT 'available',
                created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Создание таблицы групп оборудования
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS equipment_groups (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                name VARCHAR(255) NOT NULL,
                description TEXT,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Связь оборудования с группами
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS equipment_group_items (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                group_id UUID NOT NULL REFERENCES equipment_groups(id) ON DELETE CASCADE,
                equipment_id UUID NOT NULL REFERENCES equipment(id) ON DELETE CASCADE,
                quantity INTEGER NOT NULL DEFAULT 1,
                UNIQUE(group_id, equipment_id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Создание таблицы сквадов (для разделения кабинетов и зон ответственности)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS squads (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                name VARCHAR(255) NOT NULL,
                description TEXT,
                location VARCHAR(255),
                responsible_user_id UUID REFERENCES users(id),
                created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Добавление поля squad_id в equipment (если колонки ещё нет)
        let _ = sqlx::query(
            "DO $$ BEGIN ALTER TABLE equipment ADD COLUMN squad_id UUID REFERENCES squads(id); EXCEPTION WHEN duplicate_column THEN NULL; END $$"
        )
        .execute(&self.pool)
        .await;

        // Добавление поля squad_id в equipment_categories (NULL = общая категория)
        let _ = sqlx::query(
            "DO $$ BEGIN ALTER TABLE equipment_categories ADD COLUMN squad_id UUID REFERENCES squads(id); EXCEPTION WHEN duplicate_column THEN NULL; END $$"
        )
        .execute(&self.pool)
        .await;

        // Создание таблицы бронирований
        sqlx::query(
            r#"
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
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Создание таблицы разрешений
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS permissions (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                booking_id UUID NOT NULL REFERENCES bookings(id) ON DELETE CASCADE,
                equipment_id UUID REFERENCES equipment(id),
                permission_type VARCHAR(50) NOT NULL,
                issued_by UUID REFERENCES users(id),
                issued_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                expires_at TIMESTAMP WITH TIME ZONE,
                status VARCHAR(50) DEFAULT 'active'
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Создание таблицы журнала действий
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS activity_logs (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                user_id UUID REFERENCES users(id),
                action VARCHAR(100) NOT NULL,
                entity_type VARCHAR(50) NOT NULL,
                entity_id UUID,
                details JSONB,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Создание таблицы уведомлений
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS notifications (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                user_id UUID NOT NULL REFERENCES users(id),
                title VARCHAR(255) NOT NULL,
                message TEXT NOT NULL,
                notification_type VARCHAR(50) NOT NULL,
                is_read BOOLEAN DEFAULT FALSE,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Создание таблицы перемещений оборудования
        sqlx::query(
            r#"
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
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        // Создание таблицы устройств для push-уведомлений
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS user_devices (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                fcm_token TEXT NOT NULL,
                platform TEXT,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Миграция: добавить is_unique в существующие таблицы equipment (если колонки ещё нет)
        let _ = sqlx::query(
            "DO $$ BEGIN ALTER TABLE equipment ADD COLUMN is_unique BOOLEAN NOT NULL DEFAULT false; EXCEPTION WHEN duplicate_column THEN NULL; END $$"
        )
        .execute(&self.pool)
        .await;

        // Создание индексов для оптимизации
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_equipment_category ON equipment(category_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_equipment_qr ON equipment(qr_code)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_bookings_user ON bookings(user_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_bookings_equipment ON bookings(equipment_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_bookings_status ON bookings(status)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_activity_logs_user ON activity_logs(user_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_notifications_user ON notifications(user_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE UNIQUE INDEX IF NOT EXISTS user_devices_user_token_idx ON user_devices(user_id, fcm_token)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_user_devices_user ON user_devices(user_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_squads_responsible ON squads(responsible_user_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_equipment_squad ON equipment(squad_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_equipment_categories_squad ON equipment_categories(squad_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_equipment_movements_equipment ON equipment_movements(equipment_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_equipment_movements_moved_at ON equipment_movements(moved_at)")
            .execute(&self.pool)
            .await?;

        // Создание администратора по умолчанию, если его нет
        self.create_default_admin().await?;

        Ok(())
    }

    async fn create_default_admin(&self) -> Result<(), sqlx::Error> {
        use bcrypt::{hash, DEFAULT_COST};
        
        let admin_exists: Option<(bool,)> = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM users WHERE username = 'admin')"
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some((true,)) = admin_exists {
            return Ok(());
        }

        let password_hash = hash("admin123", DEFAULT_COST).unwrap();
        
        sqlx::query(
            r#"
            INSERT INTO users (username, password_hash, email, full_name, role)
            VALUES ('admin', $1, 'admin@kvantoriym.ru', 'Администратор', 'admin')
            ON CONFLICT (username) DO NOTHING
            "#,
        )
        .bind(password_hash)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

