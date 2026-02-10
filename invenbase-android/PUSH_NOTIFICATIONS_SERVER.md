# Настройка Push-уведомлений на сервере

## ⚠️ ВАЖНО: Legacy FCM API отключен

**Legacy FCM API** (`https://fcm.googleapis.com/fcm/send`) был **отключен в июле 2024 года**. Если вы получаете ошибку `404 Not Found` при отправке push-уведомлений, это означает, что нужно использовать **HTTP v1 API**.

## Текущая ситуация

Android-приложение уже настроено для получения push-уведомлений:
- ✅ Firebase Cloud Messaging интегрирован
- ✅ Токен получается и логируется
- ✅ Приложение подписывается на топик "all"
- ✅ `MyFirebaseMessagingService` обрабатывает входящие сообщения
- ✅ Токен успешно регистрируется на сервере (`POST /api/push/register-token` работает)

**Проблема**: При отправке push-уведомлений сервер получает `404 Not Found` от FCM, что означает, что Legacy API отключен и нужно использовать HTTP v1 API.

## Что нужно реализовать на сервере

### 1. Endpoint для регистрации токена

**POST `/api/push/register-token`**

**Headers:**
```
Authorization: Bearer <jwt_token>
Content-Type: application/json
```

**Body:**
```json
{
  "token": "dhaEFT59Q8KSI4Vk-o_x5m:APA91bFhXfnajzxdo_VXnbz9OkyoqHLruL0JCbJONT8c1oCEhaVBODcIG2zc1MlArN0UmLx7OL7OGNIJAPtswLb0f7YgvZ-sddChkAdKStZR0px0ROMAIh0",
  "platform": "android"
}
```

**Что делать:**
1. Извлечь `user_id` из JWT токена (как в других защищённых endpoints)
2. Сохранить FCM токен в таблице, связанной с пользователем:
   ```sql
   CREATE TABLE user_devices (
     id UUID PRIMARY KEY,
     user_id UUID REFERENCES users(id),
     fcm_token TEXT NOT NULL,
     platform TEXT, -- 'android' или 'web'
     created_at TIMESTAMP DEFAULT NOW(),
     updated_at TIMESTAMP DEFAULT NOW(),
     UNIQUE(user_id, fcm_token)
   );
   ```
3. При обновлении токена (повторный запрос с тем же user_id) обновлять `updated_at`
4. Возвращать `200 OK` при успехе

### 2. Отправка уведомлений при событиях

#### При одобрении/отклонении бронирования

В вашем коде, где обрабатывается `POST /api/bookings/{id}/approve` или `/reject`:

```python
# Пример на Python (адаптируйте под ваш язык)
import requests

def send_booking_notification(user_id, booking, status):
    # Получаем все FCM токены пользователя
    devices = db.query("SELECT fcm_token FROM user_devices WHERE user_id = ?", user_id)
    
    title = "Бронирование одобрено" if status == "approved" else "Бронирование отклонено"
    body = f"{booking.equipment_name}, {booking.quantity} шт., {format_date(booking.start_date)} - {format_date(booking.end_date)}"
    
    for device in devices:
        send_fcm_notification(
            token=device.fcm_token,
            title=title,
            body=body,
            data={
                "type": "notification",
                "booking_id": booking.id,
                "status": status
            }
        )
```

#### При истечении бронирования

В вашем cron job / scheduled task, который помечает бронирования как `expired`:

```python
def check_expired_bookings():
    expired = db.query("SELECT * FROM bookings WHERE end_date < NOW() AND status = 'approved'")
    
    for booking in expired:
        # Обновляем статус
        db.update("UPDATE bookings SET status = 'expired' WHERE id = ?", booking.id)
        
        # Отправляем уведомление
        send_booking_notification(
            booking.user_id,
            booking,
            "expired"
        )
```

### 3. Функция отправки FCM

```python
import requests
import json

FCM_SERVER_KEY = "YOUR_SERVER_KEY_FROM_FIREBASE_CONSOLE"
FCM_URL = "https://fcm.googleapis.com/fcm/send"

def send_fcm_notification(token, title, body, data=None):
    headers = {
        "Authorization": f"key={FCM_SERVER_KEY}",
        "Content-Type": "application/json"
    }
    
    payload = {
        "to": token,  # или "/topics/all" для массовой рассылки
        "notification": {
            "title": title,
            "body": body
        },
        "data": data or {},
        "android": {
            "priority": "high"
        }
    }
    
    response = requests.post(FCM_URL, headers=headers, json=payload)
    return response.status_code == 200
```

### Где взять FCM_SERVER_KEY (подробная инструкция)

**FCM_SERVER_KEY** — это ключ сервера Firebase Cloud Messaging, который используется для отправки push-уведомлений с вашего сервера на устройства пользователей.

#### Пошаговая инструкция:

1. **Откройте Firebase Console**
   - Перейдите на https://console.firebase.google.com/
   - Войдите в свой аккаунт Google (если не авторизованы)

2. **Выберите ваш проект**
   - В списке проектов найдите и выберите проект, к которому привязано ваше Android приложение
   - Если проекта нет, создайте новый (Project Settings → Add app → Android)

3. **Откройте настройки проекта**
   - В левом верхнем углу нажмите на иконку ⚙️ (шестерёнка) рядом с названием проекта
   - Или перейдите в меню: **Project Settings** (Настройки проекта)

4. **Перейдите в раздел Cloud Messaging**
   - В верхней части страницы настроек найдите вкладки: **General**, **Service accounts**, **Cloud Messaging**, и т.д.
   - **ВАЖНО**: Нажмите на вкладку **Cloud Messaging** (НЕ Service accounts!)
   - Если вкладки Cloud Messaging нет, прокрутите вниз на вкладке General до раздела "Cloud Messaging API (Legacy)"

5. **Найдите Server key на вкладке Cloud Messaging**
   - В разделе **Cloud Messaging API (Legacy)** вы увидите:
     - **Sender ID** — это НЕ то, что нужно (это ID отправителя)
     - **Server key** — это то, что нужно! (длинная строка, начинающаяся с `AAAA...` или похожая)
   
   ✅ **Если вы видите Server key** — скопируйте его и переходите к шагу 6.
   
   ❌ **Если Server key НЕТ** (видите только "Cloud Messaging API (V1)" или пусто):
   - **Вариант А (рекомендуется)**: Включите Legacy API:
     1. Перейдите в Google Cloud Console: https://console.cloud.google.com/
     2. Выберите ваш проект
     3. APIs & Services → Library
     4. Найдите "Firebase Cloud Messaging API (Legacy)"
     5. Включите API, если он выключен
     6. Вернитесь в Firebase Console → Project Settings → Cloud Messaging
     7. Server key должен появиться
   
   - **Вариант Б (если Legacy API недоступен)**: Используйте Service Account (см. раздел ниже "Альтернативный способ")

6. **Скопируйте Server key**
   - Нажмите на иконку копирования рядом с "Server key" или выделите и скопируйте весь ключ
   - Ключ выглядит примерно так: `AAAAxxxxxxxxxxxx:APA91bFhXfnajzxdo_VXnbz9OkyoqHLruL0JCbJONT8c1oCEhaVBODcIG2zc1MlArN0UmLx7OL7OGNIJAPtswLb0f7YgvZ-sddChkAdKStZR0px0ROMAIh0`

7. **Добавьте ключ в .env файл сервера**
   - Откройте файл `.env` в корне проекта `kvantoriym-server/`
   - Добавьте строку:
     ```
     FCM_SERVER_KEY=AAAAxxxxxxxxxxxx:APA91bFhXfnajzxdo_VXnbz9OkyoqHLruL0JCbJONT8c1oCEhaVBODcIG2zc1MlArN0UmLx7OL7OGNIJAPtswLb0f7YgvZ-sddChkAdKStZR0px0ROMAIh0
     ```
   - Замените значение на ваш реальный Server key (без кавычек!)

8. **Перезапустите сервер**
   - После добавления `FCM_SERVER_KEY` в `.env`, перезапустите Rust сервер
   - Сервер автоматически загрузит переменную окружения при старте

#### Альтернативный способ (если Server key не виден):

⚠️ **Внимание**: Этот способ сложнее и требует изменений в коде сервера. Используйте только если Legacy API недоступен.

Если в разделе Cloud Messaging вы не видите "Server key", и включить Legacy API не получается:

1. Перейдите в **Firebase Console → Project Settings → Service accounts** (вкладка, на которой вы сейчас находитесь)
2. Убедитесь, что выбран "Firebase Admin SDK" (Legacy credentials)
3. Нажмите **Generate new private key**
4. Подтвердите действие (может появиться предупреждение)
5. Скачается JSON файл (например, `invenbase-67c69-firebase-adminsdk-xxxxx.json`)

**Что делать с JSON файлом:**

Текущая реализация сервера использует простой Server key. Для использования Service Account JSON нужно:

1. **Сохраните JSON файл** в безопасном месте (например, `kvantoriym-server/firebase-service-account.json`)
2. **Добавьте в `.env`**:
   ```
   FCM_SERVICE_ACCOUNT_PATH=./firebase-service-account.json
   ```
3. **Обновите код сервера** для использования Service Account (требует библиотеку `google-cloud-messaging` или `firebase-admin`)

**⚠️ Рекомендация**: 
- **Сначала попробуйте включить Legacy API** (см. шаг 5, Вариант А выше) — это намного проще!
- Service Account нужен только если Legacy API полностью недоступен в вашем проекте

#### Проверка:

После добавления ключа в `.env`, проверьте логи сервера при запуске:
- Если ключ загружен успешно, в логах не будет ошибок
- При попытке отправить уведомление, если ключ неверный, вы увидите ошибку `401 Unauthorized` в логах FCM

### 4. Пример для Go (если используете Go)

```go
package main

import (
    "bytes"
    "encoding/json"
    "net/http"
)

const FCM_URL = "https://fcm.googleapis.com/fcm/send"
const FCM_SERVER_KEY = "YOUR_SERVER_KEY"

type FCMNotification struct {
    To           string                 `json:"to"`
    Notification FCMNotificationPayload `json:"notification"`
    Data         map[string]string      `json:"data"`
}

type FCMNotificationPayload struct {
    Title string `json:"title"`
    Body  string `json:"body"`
}

func SendFCMNotification(token, title, body string, data map[string]string) error {
    payload := FCMNotification{
        To: token,
        Notification: FCMNotificationPayload{
            Title: title,
            Body:  body,
        },
        Data: data,
    }
    
    jsonData, _ := json.Marshal(payload)
    req, _ := http.NewRequest("POST", FCM_URL, bytes.NewBuffer(jsonData))
    req.Header.Set("Authorization", "key="+FCM_SERVER_KEY)
    req.Header.Set("Content-Type", "application/json")
    
    client := &http.Client{}
    resp, err := client.Do(req)
    if err != nil {
        return err
    }
    defer resp.Body.Close()
    
    return nil
}
```

## Проверка работы

После реализации на сервере:

1. **Перезапустите Android приложение** — токен должен успешно зарегистрироваться (в логах будет `Push token registered successfully on server`)

2. **Отправьте тестовое уведомление через Firebase Console:**
   - Firebase Console → Cloud Messaging → New notification
   - Title: "Тест"
   - Text: "Проверка уведомлений"
   - Target: Topic → `all`
   - Отправить

3. **Проверьте логи Android:**
   - Должен появиться лог `FCM Message received: ...`
   - Должно появиться системное уведомление

4. **Проверьте реальные события:**
   - Одобрите/отклоните бронирование через веб-интерфейс
   - Android приложение должно получить push-уведомление

## ⚠️ Миграция на HTTP v1 API (если Legacy API не работает)

Если вы получаете ошибку `404 Not Found` при отправке push-уведомлений, это означает, что Legacy API отключен и нужно использовать **HTTP v1 API**.

### Что нужно для HTTP v1 API:

1. **Firebase Project ID**:
   - Firebase Console → Project Settings → General
   - Скопируйте "Project ID" (например, `invenbase-67c69`)

2. **Service Account JSON**:
   - Firebase Console → Project Settings → Service accounts
   - Нажмите "Generate new private key"
   - Скачайте JSON файл (например, `invenbase-67c69-firebase-adminsdk-xxxxx.json`)

3. **Добавьте в `.env`**:
   ```
   FCM_PROJECT_ID=invenbase-67c69
   FCM_SERVICE_ACCOUNT_PATH=./firebase-service-account.json
   ```

4. **Обновите код сервера** для использования HTTP v1 API (требует библиотеку `fcm_http1` или ручную реализацию с OAuth токенами)

**Примечание**: Реализация HTTP v1 API сложнее, чем Legacy API, так как требует генерации OAuth 2.0 токенов из Service Account JSON.

## Важные замечания

- **Токены могут обновляться** — при каждом `onNewToken` Android отправляет новый токен на сервер. Обновляйте запись в БД.
- **Один пользователь может иметь несколько устройств** — сохраняйте все токены, отправляйте на все.
- **Токены могут стать невалидными** — при отправке FCM может вернуть ошибку, удаляйте такие токены из БД.
- **Приложение в foreground** — уведомления всё равно приходят в `onMessageReceived`, но системное уведомление нужно показывать вручную (уже реализовано).
- **Legacy API отключен** — с июля 2024 года Legacy FCM API (`fcm/send`) больше не работает. Используйте HTTP v1 API.
