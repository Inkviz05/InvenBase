# Сервер системы учёта и бронирования оборудования "Кванториум"

Rust REST API сервер для системы учёта и бронирования оборудования технопарка "Кванториум".

## Технологии

- **Actix Web** - веб-фреймворк
- **SQLx** - асинхронный доступ к базе данных PostgreSQL
- **Serde** - сериализация/десериализация JSON
- **JWT** - аутентификация
- **bcrypt** - хэширование паролей
- **QR Code** - генерация QR-кодов

## Требования

- Rust 1.70+
- PostgreSQL 14+
- Cargo

## Установка

1. Клонируйте репозиторий
2. Установите PostgreSQL и создайте базу данных:
   ```sql
   CREATE DATABASE kvantoriym;
   ```

3. Создайте файл `.env` на основе `.env.example`:
   ```bash
   cp .env.example .env
   ```

4. Настройте `.env` файл с вашими данными:
   ```
   DATABASE_URL=postgresql://postgres:postgres@localhost:5432/kvantoriym
   HOST=127.0.0.1
   PORT=8080
   JWT_SECRET=replace-with-a-long-random-secret-minimum-32-chars
   JWT_EXPIRATION=86400
   CREATE_DEFAULT_ADMIN=true
   DEFAULT_ADMIN_USERNAME=admin
   DEFAULT_ADMIN_PASSWORD=replace-with-a-strong-initial-password
   ```

5. Установите зависимости и запустите сервер:
   ```bash
   cargo build
   cargo run
   ```

   Схема БД применяется автоматически через встроенные `sqlx` migrations из каталога `migrations/`.

   Перед переводом `CHECK` constraints из `NOT VALID` в validated проверьте старые данные:
   ```bash
   psql "$DATABASE_URL" -f sql/audit_constraints.sql
   ```

## Использование

### Первоначальная настройка

Первый администратор создаётся только если в `.env` включено `CREATE_DEFAULT_ADMIN=true`.
Используйте сильный `DEFAULT_ADMIN_PASSWORD`, войдите под этим пользователем и затем отключите seed администратора.

### Веб-интерфейс

После запуска сервера откройте в браузере:
- `http://localhost:8080` - страница входа
- `http://localhost:8080/admin` - административная панель

### API Endpoints

Все API endpoints доступны по пути `/api`:

#### Аутентификация
- `POST /api/auth/login` - вход в систему

#### Пользователи
- `GET /api/users` - список пользователей (требует auth)
- `POST /api/users` - создать пользователя (требует admin)
- `GET /api/users/me` - текущий пользователь
- `GET /api/users/{id}` - получить пользователя
- `PUT /api/users/{id}` - обновить пользователя
- `DELETE /api/users/{id}` - удалить пользователя

#### Оборудование
- `GET /api/equipment` - список оборудования
- `POST /api/equipment` - добавить оборудование
- `GET /api/equipment/{id}` - получить оборудование
- `PUT /api/equipment/{id}` - обновить оборудование
- `DELETE /api/equipment/{id}` - удалить оборудование
- `GET /api/equipment/qr/{qr_code}` - найти по QR-коду

#### Категории
- `GET /api/categories` - список категорий
- `POST /api/categories` - создать категорию
- `GET /api/categories/{id}` - получить категорию
- `PUT /api/categories/{id}` - обновить категорию
- `DELETE /api/categories/{id}` - удалить категорию

#### Бронирования
- `GET /api/bookings` - список бронирований
- `POST /api/bookings` - создать бронирование
- `GET /api/bookings/{id}` - получить бронирование
- `PUT /api/bookings/{id}` - обновить бронирование
- `DELETE /api/bookings/{id}` - удалить бронирование

#### Разрешения
- `GET /api/permissions` - список разрешений
- `POST /api/permissions` - создать разрешение
- `POST /api/permissions/{id}/revoke` - отозвать разрешение

#### Уведомления
- `GET /api/notifications` - список уведомлений
- `GET /api/notifications/count` - количество непрочитанных
- `POST /api/notifications/{id}/read` - отметить как прочитанное
- `POST /api/notifications/read-all` - отметить все как прочитанные

#### Отчёты
- `GET /api/reports/equipment` - отчёт по оборудованию
- `GET /api/reports/bookings` - отчёт по бронированиям

#### Логи
- `GET /api/logs` - журнал действий

#### QR-коды
- `GET /api/qr/{id}` - сгенерировать QR-код (PNG изображение)
- `GET /api/qr/{id}/data` - получить данные QR-кода

## Роли пользователей

- **admin** - полный доступ ко всем функциям
- **responsible** - доступ к управлению оборудованием и бронированиями
- **user** - создание бронирований и просмотр своего оборудования

## Структура базы данных

Сервер автоматически создаёт необходимые таблицы при первом запуске:
- `users` - пользователи
- `equipment` - оборудование
- `equipment_categories` - категории оборудования
- `equipment_groups` - группы оборудования
- `equipment_group_items` - связь оборудования с группами
- `bookings` - бронирования
- `permissions` - разрешения
- `activity_logs` - журнал действий
- `notifications` - уведомления

## Разработка

### Запуск в режиме разработки

```bash
cargo run
```

### Тестирование

```bash
cargo test
```

### Сборка для production

```bash
cargo build --release
```

## Безопасность

- Пароли хранятся в хэшированном виде (bcrypt)
- JWT токены для аутентификации
- Разграничение доступа по ролям
- HTTPS рекомендуется для production (настройте через reverse proxy)

## Лицензия

Проект выполняется в рамках дипломной работы.

