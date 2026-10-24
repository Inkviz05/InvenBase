# Быстрый запуск backend

## 1. PostgreSQL

Создайте пустую базу `kvantoriym` и убедитесь, что пользователь из connection string имеет к ней доступ. Таблицы вручную создавать не нужно: SQLx migrations применятся при старте.

## 2. Конфигурация

```powershell
cd D:\InvenBase\invenbase-server
Copy-Item .env.example .env
notepad .env
```

Минимальная dev-конфигурация:

```env
DATABASE_URL=postgresql://postgres:YOUR_PASSWORD@localhost:5432/kvantoriym
JWT_SECRET=replace-with-a-unique-secret-at-least-32-characters
CREATE_DEFAULT_ADMIN=true
DEFAULT_ADMIN_USERNAME=admin
DEFAULT_ADMIN_PASSWORD=replace-with-at-least-12-characters
```

Спецсимволы в логине или пароле `DATABASE_URL` должны быть percent-encoded.

## 3. Запуск

```powershell
cargo run
```

Ожидаемые адреса:

- API: `http://127.0.0.1:8080/api`
- liveness: `http://127.0.0.1:8080/health`
- readiness с проверкой PostgreSQL: `http://127.0.0.1:8080/ready`
- legacy HTML: `http://127.0.0.1:8080/` и `/admin`

React web запускается отдельно из `invenbase-client/web` на порту `3000`.

## Типичные ошибки

- `JWT_SECRET is required` — файл `.env` отсутствует или секрет короче 32 символов.
- `invalid password` при соединении — пароль PostgreSQL в `DATABASE_URL` неверен. При локализованном Windows PostgreSQL SQLx может показать это как `non-UTF-8 string for its error message`.
- Login возвращает `401`, хотя admin найден — bootstrap не меняет пароль существующего admin. Используйте прежний пароль или сбросьте `password_hash` штатно в БД.
- FCM-переменные не нужны для обычного запуска. Удалите пустые `FCM_*=` строки, чтобы текущая конфигурация не считала их заданными.

После создания первого администратора установите:

```env
CREATE_DEFAULT_ADMIN=false
```
