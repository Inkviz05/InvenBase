# Технический бэклог InvenBase

Проверено по репозиторию 23 октября 2026 года. Завершённые старые пункты удалены из списка; история доступна в Git.

## Критично перед production

1. **Нет TLS-конфигурации.** Compose публикует nginx по HTTP. Нужен внешний TLS-терминатор или отдельная production-конфигурация.
2. **Нет документированной процедуры backup/restore и отдельного migrate job.** Backend автоматически применяет migrations при старте.
3. **Нет readiness-проверки БД.** `/health` всегда отвечает `200`, если HTTP-процесс работает.
4. **Нет полного API-контракта.** OpenAPI/Swagger и contract tests отсутствуют.
5. **CI не выполняет DB integration tests.** Workflow запускает `cargo test --locked --lib`, но не тесты из `invenbase-server/tests/`.

## Высокий приоритет

1. Добавить backend HTTP integration tests для middleware, ролей и формата ошибок.
2. Нормализовать пустые FCM env и сократить диагностические логи конфигурации.
3. Убрать или официально пометить legacy FCM fallback; рабочий предпочтительный путь — HTTP v1 с service account.
4. Добавить secret scanning и запрет staged секретов.
5. Защитить Android auth storage, разделить API URL по build type и настроить release signing/minification.
6. Добавить Android build/tests в CI.
7. Добавить role guards на маршрутах web, не полагаясь только на скрытие пунктов меню и backend `403`.

## Средний приоритет

1. Продолжить разделение backend handlers и services/repositories.
2. Ввести единый документированный формат API errors.
3. Добавить structured logging, request ID и метрики.
4. Расширить web unit/e2e тесты на reports, support и полный lifecycle бронирования.
5. Разбить крупные React-страницы и внедрить lazy loading.
6. Ввести repository/ViewModel слой в Android и покрыть критичные сценарии тестами.

## Низкий приоритет

1. Систематизировать повторяющиеся web-компоненты: статусы, фильтры, таблицы и booking actions.
2. Рассмотреть TypeScript после стабилизации API-контрактов.
3. Рассмотреть абстракцию push transport только при реальной необходимости второго провайдера; RuStore/HMS в текущем коде отсутствуют.

## Подтверждённые ограничения текущей реализации

- Backend также содержит встроенные страницы `/` и `/admin` из `invenbase-server/static`; production Compose отдаёт React через nginx, поэтому эти страницы не являются основным web UI.
- PostgreSQL не опубликован наружу основным Compose-файлом.
- Android `usesCleartextTraffic=false`; локальный HTTP backend не заработает без debug network-security настройки или HTTPS.
- Android release build имеет `minifyEnabled false`.
- Автоматической ротации JWT/refresh token нет: API выдаёт один JWT с временем жизни из `JWT_EXPIRATION`.
- Отдельного logout endpoint на backend нет; клиенты удаляют локальный токен.

Приоритетный порядок: readiness и production DB-процедура → HTTP/API tests и контракт → CI с PostgreSQL → Android security/build → архитектурные рефакторинги.
