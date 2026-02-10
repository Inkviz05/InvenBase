# Настройка Firebase Cloud Messaging для Push-уведомлений

## Шаги настройки:

1. **Создайте проект в Firebase Console:**
   - Перейдите на https://console.firebase.google.com/
   - Создайте новый проект или выберите существующий
   - Добавьте Android приложение с package name: `com.invenbase.app`

2. **Скачайте google-services.json:**
   - В Firebase Console перейдите в Project Settings → Your apps
   - Скачайте файл `google-services.json`
   - Поместите его в папку `app/` (на том же уровне, что и `build.gradle`)

3. **Проверьте зависимости:**
   - Убедитесь, что в `build.gradle` (project level) добавлен плагин:
     ```gradle
     id 'com.google.gms.google-services' version '4.4.0' apply false
     ```
   - В `app/build.gradle` добавлен:
     ```gradle
     id 'com.google.gms.google-services'
     ```
   - И зависимости Firebase уже добавлены

4. **Синхронизируйте проект:**
   - В Android Studio: File → Sync Project with Gradle Files

5. **Проверьте работу:**
   - При первом запуске приложение автоматически подпишется на топик "all"
   - Токен FCM будет логироваться в Logcat с тегом "FCM"

## Отправка уведомлений:

Уведомления можно отправлять через:
- Firebase Console → Cloud Messaging
- Backend API (используя FCM Server Key)
- Топик "all" для массовых уведомлений

Приложение автоматически откроет экран уведомлений при клике на push-уведомление.
