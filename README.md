# InvenBase

InvenBase — система учёта, перемещения и бронирования оборудования для технопарка «Кванториум».

## Состав репозитория

```text
invenbase-server/       Rust 2021, Actix Web, SQLx, PostgreSQL
invenbase-client/web/   React 18, Vite, React Router
invenbase-android/      Native Android-клиент на Java
scripts/                PowerShell-сценарии тестирования
.github/workflows/      CI для quality checks и Docker smoke
```

Старого WebView-клиента в актуальной архитектуре нет. Поддерживаются React web и native Android.

## Быстрый локальный запуск

### Backend

Нужны Rust/Cargo и PostgreSQL. Создайте базу `kvantoriym`, затем:

```powershell
cd invenbase-server
Copy-Item .env.example .env
# отредактируйте DATABASE_URL, JWT_SECRET и параметры первого администратора
cargo run
```

`JWT_SECRET` обязателен и должен содержать не менее 32 символов. Если `CREATE_DEFAULT_ADMIN=true`, обязателен `DEFAULT_ADMIN_PASSWORD` длиной не менее 12 символов. Администратор создаётся только при отсутствии пользователя с таким именем; существующий пароль не перезаписывается.

Backend применяет SQLx migrations при старте. API доступен на `http://127.0.0.1:8080/api`, liveness — на `/health`, readiness с проверкой PostgreSQL — на `/ready`.

### Web

Нужны Node.js и npm:

```powershell
cd invenbase-client\web
Copy-Item .env.example .env
npm ci
npm run dev
```

Vite запускается на `http://localhost:3000`. В dev-режиме `/api` проксируется на backend; порт backend задаётся через `VITE_API_PORT`.

### Docker Compose

Нужен запущенный Docker Desktop с доступной командой `docker`:

```powershell
Copy-Item .env.docker.example .env
# задайте POSTGRES_PASSWORD, JWT_SECRET и DEFAULT_ADMIN_PASSWORD
docker compose up --build
```

Единая точка входа — `http://localhost` или порт из `WEB_PORT`. PostgreSQL и backend наружу не публикуются. Подробнее: [DOCKER_DEPLOYMENT.md](DOCKER_DEPLOYMENT.md).

### Native Android

Проект требует Android SDK 34 и JDK 17 или 21:

```powershell
cd invenbase-android
.\gradlew.bat assembleDebug
```

APK: `invenbase-android/app/build/outputs/apk/debug/app-debug.apk`. Текущий URL API по умолчанию задан в `invenbase-android/app/src/main/java/com/invenbase/app/Config.java` и может быть переопределён сохранённой настройкой приложения.

## Реализованная функциональность

- JWT-аутентификация и роли `admin`, `responsible`, `user`.
- Пользователи, категории, оборудование, перемещения, группы и отряды.
- Одиночные и пакетные бронирования; approve, reject, cancel, return и автоматическое истечение.
- QR-коды и сканирование.
- In-app уведомления, multi-device реестр и отправка push через FCM HTTP v1, RuStore и Huawei Push Kit; Android-клиент регистрирует токены всех трёх providers. Legacy FCM оставлен как fallback в коде.
- Журнал действий, отчёты и экспорт XLSX в web.
- Заявки поддержки и сообщения в них.

## Проверки

```powershell
.\scripts\test-backend.ps1

cd invenbase-client\web
npm run test:run
npm run build

cd ..\..
.\scripts\test-docker-compose-smoke.ps1
```

Полная тестовая матрица описана в [TESTING.md](TESTING.md), реальные browser smoke-сценарии — в [REAL_SMOKE_TESTING.md](REAL_SMOKE_TESTING.md).

## Важные ограничения

- `/health` проверяет HTTP-процесс, а `/ready` возвращает `200` только при доступной PostgreSQL.
- OpenAPI/Swagger в репозитории нет.
- CI собирает Android debug APK, запускает Rust library tests, полный backend DB/HTTP integration suite на чистом PostgreSQL 16, web tests/build/audit и Docker browser smoke.
- Release Android собирается без минификации (`minifyEnabled false`), API URL не разделён по build flavors.
- Миграции запускаются самим backend; отдельной production-команды migrate/rollback нет.

## Документация

- [Backend](invenbase-server/README.md) и [его быстрый старт](invenbase-server/QUICKSTART.md)
- [Web-клиент](invenbase-client/web/README.md)
- [Docker deployment](DOCKER_DEPLOYMENT.md)
- [Тестирование](TESTING.md)
- [Native Android](invenbase-android/README.md), [сборка](invenbase-android/BUILD_INSTRUCTIONS.md), [Firebase](invenbase-android/FIREBASE_SETUP.md), [push backend](invenbase-android/PUSH_NOTIFICATIONS_SERVER.md)
- [Текущий план работ](PLAN_DORABOTOK.md)
- [Актуальный технический бэклог](PROJECT_WEAKNESSES_BACKLOG.md)

## Production checklist

- Использовать уникальные `POSTGRES_PASSWORD`, `JWT_SECRET` и пароль первого администратора.
- После создания администратора установить `CREATE_DEFAULT_ADMIN=false`.
- Ограничить `CORS_ALLOWED_ORIGINS` реальными web-origin.
- Не коммитить `.env`, `google-services.json` и Firebase service account.
- Использовать HTTPS перед публичным доступом; Compose-конфигурация сама TLS не настраивает.
- Настроить резервное копирование PostgreSQL и проверить восстановление до обновления production.
