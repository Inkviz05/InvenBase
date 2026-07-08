# Слабые места проекта и список доработок

Дата аудита: 2026-07-08.

## Критично

1. Нет Docker/Compose окружения для воспроизводимого запуска.
   - Добавить `Dockerfile` для `invenbase-server` с multi-stage Rust build.
   - Добавить `Dockerfile` для web: `npm run build` + nginx/caddy для статических файлов.
   - Добавить `docker-compose.yml` для `postgres`, `server`, `web`.
   - Добавить reverse proxy в compose: `nginx` или `caddy` как единую точку входа.
   - Через reverse proxy отдавать web на `/`, проксировать `/api` на backend, `/health` и `/ready` оставлять доступными для healthchecks.
   - Для production-профиля добавить TLS: Caddy с автоматическим HTTPS или nginx с mounted certificates.
   - Добавить лимиты reverse proxy: `client_max_body_size`, таймауты, gzip/brotli, security headers.
   - Добавить `.dockerignore` для Rust, web и Android артефактов.
   - Добавить healthchecks: Postgres `pg_isready`, backend `/api/health` или отдельный `/health`.
   - Вынести production env в `.env.example`, реальные секреты не хранить в репозитории.

2. Нет CI-пайплайна.
   - Добавить GitHub Actions или аналог: `cargo test`, `cargo build`, `npm run build`, Android `assembleDebug`.
   - Отдельно проверять SQLx migrations на чистой PostgreSQL.
   - Добавить проверку отсутствия секретов и `.env` в staged files.

3. Тесты покрывают в основном бронирования, но не всё приложение.
   - Backend: расширить unit/integration tests на auth, users, equipment, categories, groups/squads, permissions, notifications, support requests, reports, QR и logs.
   - Backend API: добавить HTTP integration tests через Actix test server, чтобы проверять не только service layer, но и реальные routes, middleware, auth roles и формат ошибок.
   - База данных: поднимать чистую PostgreSQL для тестов, прогонять SQLx migrations, наполнять минимальные fixtures и проверять ограничения/индексы.
   - Web: добавить Vitest + React Testing Library для критичных компонентов и страниц: login, equipment list/detail, booking create/list, reports, support.
   - Web E2E: добавить Playwright smoke tests на основные сценарии: вход, создание оборудования, создание бронирования, approve/reject/cancel/return.
   - Android unit tests: покрыть `AuthManager`, `ApiErrorParser`, adapters, модели и логику фильтров/статусов.
   - Android instrumentation tests: Espresso smoke для входа, списка оборудования, бронирований и подтверждения возврата.
   - Contract tests: зафиксировать API-контракты между backend, React и Android через OpenAPI или JSON fixtures.
   - Docker/Compose tests: добавить smoke-проверку запуска всей системы в контейнерах: Postgres healthy, backend ready, web отдаёт UI, `/api` проксируется.
   - CI: разделить быстрые проверки на каждый push и полный набор e2e/instrumentation tests для release/nightly.

4. Production-настройки backend пока слишком dev-oriented.
   - `env_logger` по умолчанию запускается с `debug`; для production нужен `info`/`warn`.
   - `actix_files::Files::new("/static", "./static").show_files_listing()` включает directory listing. В production listing нужно убрать.
   - `HOST` по умолчанию `127.0.0.1`; для Docker нужен `0.0.0.0`.
   - Pool PostgreSQL сейчас `max_connections(10)` и timeout захардкожены; вынести в env.

5. Нет явного healthcheck/readiness API.
   - Добавить `GET /health` без авторизации.
   - Добавить `GET /ready`, который проверяет доступность БД.
   - Использовать эти endpoints в Docker Compose, reverse proxy и мониторинге.

## Высокий приоритет

6. Мобильную разработку нужно перепланировать.
   - Native Android Java временно оставить только в режиме поддержки критичных исправлений.
   - Новую мобильную разработку планировать на Flutter, но не начинать до стабилизации backend и web.
   - При проектировании backend API учитывать будущий Flutter-клиент: стабильные DTO, единый формат ошибок, OpenAPI, pagination/filtering.
   - Не вкладываться сейчас в глубокий Android Java refactoring/MVVM, кроме исправлений, которые напрямую нужны пользователям.
   - Push-стратегию проектировать провайдер-независимой, чтобы потом подключить Flutter-клиент без переписывания backend.

7. Секреты и окружение требуют дисциплины.
   - В рабочей копии есть реальные `.env`; `.gitignore` их игнорирует, но нужен `README` с правилом ротации секретов.
   - Добавить `scripts/check-secrets` или pre-commit hook.
   - Для FCM service account использовать volume/secret, а не файл внутри образа.

8. Нет reverse proxy конфигурации.
   - Добавить пример `nginx.conf` или `Caddyfile`.
   - HTTPS, gzip/brotli, лимиты body size, проксирование `/api` на backend.
   - Отдельно задать security headers для web.
   - Для локального Docker Compose сделать профиль `dev` без TLS и профиль `prod` с TLS.
   - Развести internal network (`postgres`, `server`) и public network (`reverse-proxy`, `web`).
   - Запретить прямую публикацию порта backend наружу в production compose.

9. Нужно уйти от Google Firebase/FCM или подготовить независимый fallback.
   - Текущий Android использует Firebase Messaging и `google-services` plugin, backend содержит FCM HTTP v1/legacy логику.
   - Российский кандидат N1: RuStore Push SDK. В документации RuStore указано, что сервис является альтернативой Firebase Cloud Messaging; есть Push SDK для Kotlin/Java и API отправки уведомлений.
   - Ссылка: https://www.rustore.ru/help/sdk/push-notifications
   - Плюсы RuStore: российская экосистема, Android SDK, серверный API отправки, Universal Push SDK с несколькими каналами доставки.
   - Минусы RuStore: доставка зависит от RuStore/приложений-дистрибьюторов экосистемы VK и авторизации пользователя в приложении-дистрибьюторе; нужно отдельно проверить покрытие устройств аудитории.
   - Кандидат N2: AppMetrica Push SDK. Есть Android Push SDK, но в документации транспорт настраивается через Firebase или HMS, поэтому это не полный отказ от Google само по себе.
   - Ссылка: https://appmetrica.yandex.ru/docs/ru/sdk/android/push/quick-start
   - Собственная система уведомлений: оставить in-app notifications в БД, добавить WebSocket/SSE канал для web и Android foreground, а для фоновых мобильных push использовать RuStore/HMS/FCM как внешние транспорты.
   - Рекомендуемый путь: ввести абстракцию `NotificationTransport` на backend (`in_app`, `rustore`, `fcm`, позже `hms`), хранить токены с типом провайдера, постепенно отключить Firebase после проверки RuStore на реальных устройствах.
   - Для Android: заменить прямую зависимость от Firebase на слой `PushProvider`, добавить RuStore provider, оставить Firebase provider временно за build flavor/feature flag.
   - Для Docker: секреты push-провайдеров передавать через env/secret volumes, не вкладывать service account или API keys в образ.

10. Frontend не имеет quality gates.
   - В `package.json` есть только `dev/build/preview`.
   - Добавить ESLint, форматирование, хотя бы smoke/unit tests для критичных экранов бронирования.
   - Уменьшить bundle warning: сейчас Vite предупреждает о JS chunk > 500 kB. Разнести тяжелые страницы/`xlsx`/QR на dynamic imports.

11. Android release build не защищен и не оптимизирован.
   - Пункт не развивать глубоко до решения по Flutter; оставить как risk note для текущего Java-клиента.
   - `minifyEnabled false` в release; включить R8/ProGuard после проверки правил.
   - API URL захардкожен в `Config.java`; лучше разделить debug/release через `BuildConfig`.
   - Добавить product flavors или buildConfigField для dev/prod API.
   - Добавить базовые unit tests для adapters/utils и instrumentation smoke для auth/bookings.

## Средний приоритет

12. Backend handler/service слой еще не полностью разделен.
   - Продолжить вынос SQL из handlers в service/repository слой.
   - Унифицировать уведомления и push payloads после уже созданного `booking_events`.
   - Добавить транзакции там, где бизнес-действие меняет несколько сущностей.

13. Ошибки API и контракты не формализованы.
    - Описать OpenAPI/Swagger схему.
    - Ввести единый формат ошибок: `code`, `message`, `details`.
    - Сгенерировать/синхронизировать API-клиенты для web/android или хотя бы добавить contract tests.

14. Миграции требуют production-процедуры.
    - Сейчас migrations запускаются автоматически при старте backend. Для production лучше иметь отдельный `migrate` job/command.
    - Добавить инструкцию rollback/backup перед миграциями.
    - Валидировать текущие `NOT VALID` constraints после аудита старых данных.

15. Наблюдаемость слабая.
    - Добавить structured logs (`tracing`) вместо разрозненного `log`.
    - Добавить request id/correlation id.
    - Добавить метрики: количество бронирований по статусам, ошибки push, latency API, ошибки БД.

16. Файловая/статическая отдача backend смешана с API.
    - Для production лучше отдавать React web через nginx/caddy, backend оставить только API.
    - Старую `static/admin.html` определить как legacy/debug или убрать из production.

## Низкий приоритет

17. Документация запуска расходится с реальным состоянием.
    - В корневом README есть ссылки на файлы, которых может не быть или которые устарели.
    - Добавить разделы: local dev, Docker dev, production deploy, backup/restore.

18. Android архитектура постепенно требует MVVM.
    - Этот пункт отложен, если принято решение переходить на Flutter.
    - Сейчас Fragment/Adapter содержит много UI-логики.
    - Вынести API calls во ViewModel/Repository.
    - Добавить LiveData/StateFlow-подобный слой состояния.

19. Web UI нуждается в систематизации компонентов.
    - Фильтры, карточки, статусы и кнопки сейчас во многом inline.
    - Вынести `StatusBadge`, `BookingActions`, `FilterTabs`.
    - Это упростит дальнейший дизайн под "Кванториум" и неоновую тему.

## Предлагаемый порядок работ

1. Добавить Docker Compose для локального запуска: Postgres + backend + web.
2. Добавить `/health` и `/ready` в backend.
3. Добавить backend Dockerfile и `.dockerignore`.
4. Добавить web Dockerfile и reverse proxy (`nginx` или `caddy`) с маршрутизацией `/` и `/api`.
5. Расширить тестовую стратегию с упором на backend API и React web; Android Java покрывать только smoke/регрессионно до перехода на Flutter.
6. Добавить CI: backend tests/build и web build как обязательные; Android assembleDebug оставить отдельной необязательной проверкой до решения по Flutter.
7. Добавить Docker/Compose smoke tests для полной системы.
8. Ввести OpenAPI/единый формат ошибок, чтобы подготовить API для React web и будущего Flutter-клиента.
9. Убрать production-risk настройки: debug logs, static listing, hardcoded pool.
10. Продолжить разделение backend handlers/services.
11. Ввести abstraction layer для push: `NotificationTransport`, `PushProvider`, хранение provider-specific tokens.
12. После стабилизации backend/web сделать отдельный план Flutter-клиента и PoC push-провайдера.
