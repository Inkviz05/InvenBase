# InvenBase

Система учета, бронирования и контроля оборудования для технопарка "Кванториум".

## Структура проекта

```text
invenbase-server/       Rust / Actix Web REST API, PostgreSQL
invenbase-client/web/   React / Vite web-клиент
invenbase-android/      Native Android-приложение на Java
```

Android WebView-клиент из `invenbase-client/android` выведен из активной разработки. Основные клиентские приложения теперь: React web и native Android Java.

## Быстрый старт

### 1. Сервер

Требования:

- Rust 1.70+
- PostgreSQL 14+

```bash
cd invenbase-server
cp .env.example .env
cargo run
```

Перед запуском отредактируйте `invenbase-server/.env`:

```env
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/kvantoriym
HOST=127.0.0.1
PORT=8080
JWT_SECRET=change-me-use-a-long-random-secret-minimum-32-chars
JWT_EXPIRATION=86400
CREATE_DEFAULT_ADMIN=true
DEFAULT_ADMIN_USERNAME=admin
DEFAULT_ADMIN_PASSWORD=replace-with-a-strong-initial-password
CORS_ALLOWED_ORIGINS=http://localhost:3000
```

API будет доступен по адресу `http://localhost:8080/api`.

### 2. Web-клиент

Требования:

- Node.js 18+
- npm 9+

```bash
cd invenbase-client/web
cp .env.example .env
npm install
npm run dev
```

По умолчанию Vite запускается на `http://localhost:3000` и проксирует `/api` на backend.

### 3. Native Android

Требования:

- Android Studio
- JDK 17+
- Android SDK API 24+

```bash
cd invenbase-android
./gradlew assembleDebug
```

Для эмулятора backend должен быть доступен с устройства. В native Android используется собственная конфигурация API URL в `invenbase-android/app/src/main/java/com/invenbase/app/Config.java`.

## Основная функциональность

- Авторизация пользователей.
- Роли `admin`, `responsible`, `user`.
- Учет оборудования и категорий.
- Группы оборудования и отряды/зоны ответственности.
- Создание и обработка бронирований.
- QR-коды и QR-сканирование.
- Уведомления и push-уведомления.
- Журнал действий и отчеты.
- Заявки в техническую поддержку.

## Безопасность

Перед production-запуском обязательно:

1. Не использовать дефолтные учетные данные администратора: задавать первого администратора через `CREATE_DEFAULT_ADMIN` и `DEFAULT_ADMIN_PASSWORD`, затем отключать seed.
2. Использовать уникальный `JWT_SECRET` длиной минимум 32 символа.
3. Ограничить `CORS_ALLOWED_ORIGINS` реальными доменами клиентов.
4. Не хранить `.env`, Firebase service account и приватные ключи в git.
5. Перевыпустить FCM/Firebase ключи, если они уже попадали в репозиторий.
6. Использовать HTTPS.

## Документация

- [План доработок](PLAN_DORABOTOK.md)
- [Сервер](invenbase-server/README.md)
- [Быстрый старт сервера](invenbase-server/QUICKSTART.md)
- [Web-клиент](invenbase-client/README.md)
- [Native Android](invenbase-android/README.md)
- [Сборка Native Android](invenbase-android/BUILD_INSTRUCTIONS.md)
- [Firebase для Native Android](invenbase-android/FIREBASE_SETUP.md)
- [Push-уведомления](invenbase-android/PUSH_NOTIFICATIONS_SERVER.md)

## Ближайшие технические задачи

1. Запустить SQL-аудит старых данных и затем перевести `CHECK` constraints из `NOT VALID` в validated.
2. Расширить интеграционные тесты на роли, пользователей и support requests.
3. Разделить backend handlers и бизнес-логику на service layer.
4. Убрать debug-логи из web-клиента.
5. Постепенно привести native Android к MVVM-структуре.
