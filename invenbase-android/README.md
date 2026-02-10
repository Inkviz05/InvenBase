# InvenBase Android App

Нативное Android приложение для управления инвентарём, созданное на основе веб-версии InvenBase.

## Технологии

- **Java** - основной язык программирования
- **Gradle** - система сборки
- **Retrofit** - для HTTP запросов
- **Gson** - для работы с JSON
- **Material Design Components** - для UI
- **ZXing** - для сканирования QR-кодов
- **Navigation Component** - для навигации между экранами

## Структура проекта

```
app/
├── src/main/
│   ├── java/com/invenbase/app/
│   │   ├── api/          # API клиенты (Retrofit)
│   │   ├── fragments/    # Фрагменты экранов
│   │   ├── models/       # Модели данных
│   │   └── utils/        # Утилиты (AuthManager и т.д.)
│   ├── res/
│   │   ├── layout/       # XML разметки
│   │   ├── values/       # Ресурсы (strings, colors, themes)
│   │   └── menu/         # Меню
│   └── AndroidManifest.xml
└── build.gradle
```

## Основные функции

- ✅ Авторизация пользователей
- ✅ Просмотр оборудования
- ✅ Создание и управление бронированиями
- ✅ Сканирование QR-кодов
- ✅ Уведомления
- ✅ Dashboard со статистикой
- 🔄 Админ функции (категории, пользователи, отчёты, логи) - в разработке

## Настройка

1. Убедитесь, что у вас установлен Android Studio
2. Откройте проект в Android Studio
3. Настройте API URL в `Config.java`:
   - Для эмулятора: `http://10.0.2.2:8080/api`
   - Для реального устройства: `http://YOUR_IP:8080/api`

## Сборка

```bash
./gradlew assembleDebug
```

## Зависимости

Все зависимости указаны в `app/build.gradle`. Основные:
- AndroidX библиотеки
- Retrofit 2.9.0
- Material Design Components
- ZXing для QR-сканирования
