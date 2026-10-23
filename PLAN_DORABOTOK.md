# План доработок InvenBase

Актуализировано по исходному коду 23 октября 2026 года. Это план, а не описание уже существующих возможностей.

## Что уже сделано

- Удалён старый Android WebView-клиент; активны React web и native Android Java.
- Секреты вынесены в env, `JWT_SECRET` и bootstrap-пароль валидируются.
- Добавлены SQLx migrations, ограничения и внешние ключи PostgreSQL.
- Основная логика жизненного цикла бронирования вынесена в services и покрыта unit/DB integration tests.
- Добавлены Dockerfiles, Compose, nginx reverse proxy, healthchecks и browser smoke полного стека.
- Добавлен GitHub Actions workflow: web audit/tests/build, `cargo fmt`, backend library tests и Compose smoke.
- Убраны production `console.log`, directory listing отключён, параметры пула БД вынесены в env.
- Web использует локальный XLSX writer вместо зависимости `xlsx`.

## Следующие задачи

### 1. Backend API и безопасность

1. Добавить HTTP integration tests через `actix_web::test` для login, ролей, users, equipment и support.
2. Унифицировать JSON-ошибки полями `code`, `message`, `details`; сейчас публичной спецификации нет.
3. Добавить OpenAPI и проверку контрактов для React и Android.
4. Добавить `/ready`, который реально выполняет запрос к PostgreSQL; `/health` сейчас возвращает статический `200`.
5. Убрать подробное диагностическое логирование FCM-конфигурации и нормализовать пустые FCM env как `None`.

### 2. База данных и эксплуатация

1. Не использовать основную БД для integration tests: создавать отдельную БД и задавать `TEST_DATABASE_URL`.
2. Добавить явную production-процедуру backup, migrate и restore. Сейчас migrations выполняются при запуске backend.
3. Проверить исторические данные перед валидацией оставшихся `NOT VALID` constraints.
4. Добавить CI job с чистой PostgreSQL для полного `cargo test --locked`.

### 3. Архитектура backend

1. Продолжить перенос SQL и бизнес-правил из handlers в service/repository слой.
2. Добавить транзакции для оставшихся операций, изменяющих несколько сущностей.
3. Перейти от `log` к структурированному `tracing`, добавить request/correlation ID.
4. Добавить метрики ошибок БД, push и времени ответа API.

### 4. Web

1. Добавить route-level role guards; сейчас маршруты защищены фактом входа, а окончательная авторизация выполняется backend и частично UI.
2. Расширить Vitest-покрытие reports, support, equipment detail/edit и действий с бронированием.
3. Расширить Playwright flow на approve/reject/cancel/return и экспорт отчётов.
4. Разбить крупные страницы и добавить lazy loading; отдельного lint script сейчас нет.

### 5. Native Android

1. Убрать токены и пользовательские данные из обычных `SharedPreferences` в защищённое хранилище.
2. Вынести API URL в build types/flavors вместо одной константы и пользовательской настройки.
3. Ввести repository/ViewModel слой и сократить API/UI-логику в Activity/Fragment.
4. Добавить unit и instrumentation tests; сейчас Gradle содержит зависимости JUnit/Espresso, но тестовых исходников в проекте нет.
5. Проверить release signing, включить и протестировать R8/ProGuard.

### 6. Delivery

1. Добавить Android build в CI.
2. Добавить secret scanning и проверку случайно staged `.env`/service-account файлов.
3. Определить production TLS и reverse proxy перед публичным развёртыванием; текущий Compose — production-shaped локальный стек без TLS.

## Не планируется без отдельного решения

- Переход на микросервисы.
- Переписывание backend на другом языке.
- Полная замена native Android на Flutter/React Native.
- Большой UI-редизайн до фиксации API-контрактов и тестового покрытия.
