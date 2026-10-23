# Push-уведомления: фактическая серверная реализация

## Поток данных

1. Android получает FCM token.
2. После login `PushManager` отправляет token, стабильный `device_id`, provider `fcm` и platform `android` на `POST /api/push/register-token` с Bearer JWT.
3. Backend сохраняет устройство независимо от наличия credentials транспорта; повторная регистрация обновляет token, `active` и `last_seen_at`.
4. In-app notification и outbox-задание создаются одной транзакцией; worker доставляет событие на все активные FCM/RuStore устройства.
5. Payload содержит `notification_id`, а полное уведомление доступно через `/api/notifications`.
6. WorkManager каждые 15 минут сверяет непрочитанные уведомления и показывает те, чьи `notification_id` не были обработаны push-сервисами. После `onDeletedMessages` FCM/RuStore запускается немедленная сверка.

## Предпочтительная конфигурация: HTTP v1

```env
FCM_PROJECT_ID=your-project-id
FCM_SERVICE_ACCOUNT_PATH=C:\secure\service-account.json
```

На Linux/Docker укажите путь, доступный внутри backend container, и смонтируйте секретный файл volume/secret. Текущий `docker-compose.yml` не монтирует service account автоматически — это нужно добавить в конкретное deployment-окружение.

Backend вручную:

- читает `client_email` и `private_key` из JSON;
- подписывает OAuth assertion;
- получает token через `https://oauth2.googleapis.com/token`;
- отправляет каждое сообщение через FCM HTTP v1.

## Legacy fallback

Переменная `FCM_SERVER_KEY` и вызов `https://fcm.googleapis.com/fcm/send` ещё присутствуют в коде. Этот API отключён Google с июля 2024 года. Используйте HTTP v1; legacy ветку следует считать совместимостью со старым кодом, а не рекомендованной настройкой.

## RuStore transport

```env
RUSTORE_PROJECT_ID=your-rustore-project-id
RUSTORE_SERVICE_TOKEN=your-service-token
```

Backend отправляет токены provider `rustore` через `https://vkpns.rustore.ru/v1/projects/{project_id}/messages:send`. Ответ `NOT_FOUND` деактивирует токен. Android использует RuStore Push SDK 7.5.0, получает token при старте и через `onNewToken`, сохраняет его до авторизации и регистрирует с provider `rustore`. Для инициализации задайте Gradle property или переменную окружения `RUSTORE_PROJECT_ID`; при пустом значении интеграция отключена.

## Реальные маршруты

- `POST /api/push/register-token` — регистрация token текущего пользователя.
- `POST /api/push/unregister-device` — деактивация конкретного provider/device при logout.
- `GET /api/notifications` — in-app уведомления.
- `GET /api/notifications/count` и `/unread-count` — число непрочитанных.
- `POST /api/notifications/{id}/read` и `/read-all` — отметка прочитанного.

Публичного API для произвольной отправки push в проекте нет.

## Ограничения

- RuStore требует отдельный проект для каждой комбинации package name и подписи; реальная доставка должна быть проверена debug/release сборками, зарегистрированными в RuStore Console.
- Huawei transport и клиентский Huawei Push Kit отсутствуют.
- Отдельных network contract tests FCM/OAuth и RuStore API пока нет.
- WorkManager использует минимальный системный интервал 15 минут и не является каналом мгновенной доставки.
- Service account rotation и secret mounting должны настраиваться вне приложения.
