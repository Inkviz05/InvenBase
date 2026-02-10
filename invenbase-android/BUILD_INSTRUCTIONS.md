# Инструкции по сборке InvenBase Android

## Требования

- Android Studio Hedgehog (2023.1.1) или новее
- JDK 17 или выше
- Android SDK (API 24+)
- Gradle 8.2+

## Первоначальная настройка

1. Откройте проект в Android Studio
2. Дождитесь синхронизации Gradle (Android Studio автоматически скачает зависимости)
3. Настройте API URL в `app/src/main/java/com/invenbase/app/Config.java`:
   - Для эмулятора: `http://10.0.2.2:8080/api`
   - Для реального устройства: `http://YOUR_COMPUTER_IP:8080/api`

## Сборка проекта

### Debug версия
```bash
./gradlew assembleDebug
```

### Release версия
```bash
./gradlew assembleRelease
```

APK файл будет находиться в: `app/build/outputs/apk/`

## Запуск на эмуляторе

1. Создайте Android Virtual Device (AVD) в Android Studio
2. Минимальная версия Android: API 24 (Android 7.0)
3. Запустите приложение через Android Studio (Shift+F10)

## Запуск на реальном устройстве

1. Включите режим разработчика на устройстве
2. Включите USB отладку
3. Подключите устройство к компьютеру
4. Убедитесь, что устройство и компьютер в одной сети Wi-Fi
5. Измените API URL в `Config.java` на IP адрес вашего компьютера
6. Запустите приложение

## Структура приложения

- **LoginActivity** - экран входа
- **MainActivity** - главный экран с Bottom Navigation
- **DashboardFragment** - главная страница со статистикой
- **EquipmentFragment** - список оборудования
- **BookingsFragment** - список бронирований
- **ScannerFragment** - сканер QR-кодов

## API интеграция

Приложение использует тот же API, что и веб-версия:
- Base URL настраивается в `Config.java`
- Авторизация через Bearer токены
- Все запросы идут через Retrofit

## Дальнейшая разработка

Для добавления новых функций:
1. Добавьте методы в `ApiService.java`
2. Создайте модели в `models/`
3. Создайте фрагменты/активности
4. Добавьте навигацию в `MainActivity`
