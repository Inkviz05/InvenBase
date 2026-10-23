# Push-уведомления: фактическая серверная реализация

## Поток данных

1. Android получает FCM token.
2. После login `PushManager` отправляет `{ "token": "..." }` на `POST /api/push/register-token` с Bearer JWT.
3. Backend сохраняет token в `user_devices` с platform `android` только если настроен FCM.
4. События бронирований, background expiration checker и support handlers вызывают отправку пользователю.
5. In-app notifications хранятся отдельно в PostgreSQL и доступны через `/api/notifications`.

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

## Реальные маршруты

- `POST /api/push/register-token` — регистрация token текущего пользователя.
- `GET /api/notifications` — in-app уведомления.
- `GET /api/notifications/count` и `/unread-count` — число непрочитанных.
- `POST /api/notifications/{id}/read` и `/read-all` — отметка прочитанного.

Публичного API для произвольной отправки push в проекте нет.

## Ограничения

- В таблице и API provider зафиксирован как FCM; RuStore/HMS providers не реализованы.
- Пустые `FCM_*=` текущий config может интерпретировать как заданные значения; незаданные переменные лучше удалить.
- Отдельных automated tests FCM/OAuth потока нет.
- Service account rotation и secret mounting должны настраиваться вне приложения.
