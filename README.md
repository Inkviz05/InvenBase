# InvenBase - Система учёта и бронирования оборудования

Полнофункциональная система управления инвентарём и бронированием оборудования для технопарка "Кванториум".

## 📋 Структура проекта

Проект состоит из трёх основных компонентов:

```
├── invenbase-server/      # Rust REST API сервер
├── invenbase-client/      # Веб-приложение (React) и Android (WebView)
└── invenbase-android/     # Нативное Android приложение (Java)
```

### Компоненты

| Компонент | Технология | Назначение |
|-----------|-----------|-----------|
| **invenbase-server** | Rust (Actix Web) + PostgreSQL | REST API, бэкенд |
| **invenbase-client** | React (Vite) | Веб-приложение и сборка для Android |
| **invenbase-android** | Java (Android Native) | Нативное Android приложение |

## 🚀 Быстрый старт (3 шага)

### 1️⃣ Запуск сервера

```bash
# Перейти в папку сервера
cd invenbase-server

# Создать файл .env
cp .env.example .env

# Запустить (требует PostgreSQL)
cargo run
```

Сервер будет доступен по адресу: **http://localhost:8080**

Данные для входа:
- **Логин**: `admin`
- **Пароль**: `admin123`

### 2️⃣ Запуск веб-приложения

```bash
# Перейти в папку клиента
cd invenbase-client/web

# Установить зависимости
npm install

# Запустить dev-сервер
npm run dev
```

Приложение будет доступно по адресу: **http://localhost:3000**

### 3️⃣ Запуск Android приложения

```bash
# Перейти в папку Android приложения
cd invenbase-android

# Собрать Debug версию
./gradlew assembleDebug

# Или запустить через Android Studio
```

---

## 📦 Полные инструкции

### Сервер (invenbase-server)

**Требования:**
- Rust 1.70+
- PostgreSQL 14+
- Git

**Подробные инструкции:**

1. **Установка PostgreSQL**
   ```bash
   # Создайте базу данных
   psql -U postgres
   CREATE DATABASE kvantoriym;
   \q
   ```

2. **Конфигурация окружения**
   ```bash
   cd invenbase-server
   cp .env.example .env
   ```
   
   Отредактируйте `.env`:
   ```env
   DATABASE_URL=postgresql://postgres:postgres@localhost:5432/kvantoriym
   HOST=127.0.0.1          # 0.0.0.0 для доступа из сети
   PORT=8080
   JWT_SECRET=your-secret-key-min-32-chars
   JWT_EXPIRATION=86400
   ```

3. **Запуск**
   ```bash
   cargo build
   cargo run
   ```

4. **Проверка**
   - Веб-интерфейс: http://localhost:8080
   - Админ-панель: http://localhost:8080/admin
   - API: http://localhost:8080/api

📖 Подробнее: [invenbase-server/README.md](invenbase-server/README.md)

### Веб-приложение (invenbase-client/web)

**Требования:**
- Node.js 18+
- npm 9+

**Установка и запуск:**

```bash
cd invenbase-client/web

# Установить зависимости
npm install

# Dev-сервер (с горячей перезагрузкой)
npm run dev

# Сборка для продакшена
npm run build

# Предпросмотр сборки
npm run preview
```

**Конфигурация API:**

Создайте файл `.env` в `invenbase-client/web/`:
```env
VITE_API_URL=http://localhost:8080/api
```

📖 Подробнее: [invenbase-client/README.md](invenbase-client/README.md)

### Android приложение (invenbase-android)

**Требования:**
- Android Studio Hedgehog (2023.1.1)+
- JDK 17+
- Android SDK API 24+

**Запуск Debug версии:**

```bash
cd invenbase-android

# Через Gradle
./gradlew assembleDebug

# Или через Android Studio: Build → Build Bundle(s) / APK(s) → Build APK(s)
```

**Конфигурация API:**

Отредактируйте `app/src/main/java/com/invenbase/app/Config.java`:
```java
// Для эмулятора
public static final String API_BASE_URL = "http://10.0.2.2:8080/api";

// Для реального устройства
public static final String API_BASE_URL = "http://YOUR_IP:8080/api";
```

📖 Подробнее: [invenbase-android/BUILD_INSTRUCTIONS.md](invenbase-android/BUILD_INSTRUCTIONS.md)

### Android из веб-приложения (invenbase-client)

Можно собрать Android APK из веб-приложения:

**Windows:**
```bash
cd invenbase-client
build-android.bat
```

**Linux/Mac:**
```bash
cd invenbase-client
chmod +x build-android.sh
./build-android.sh
```

Скрипт автоматически:
1. Собирает веб-приложение
2. Копирует файлы в Android проект
3. Готовит к сборке в Android Studio

---

## 🔧 Системные требования

### Минимальные требования

| Компонент | Требования |
|-----------|-----------|
| **invenbase-server** | Rust 1.70+, PostgreSQL 14+, 2GB RAM |
| **invenbase-client** | Node.js 18+, 2GB RAM |
| **invenbase-android** | Android 7.0+ (API 24+), 4GB RAM |

### Рекомендуемые требования

- **OS:** Windows 10/11, macOS, Linux
- **RAM:** 8GB
- **Диск:** 20GB свободного места
- **Интернет:** Для скачивания зависимостей

---

## 📱 Функциональность

### Все платформы поддерживают:
- ✅ Авторизация пользователей
- ✅ Просмотр оборудования
- ✅ Создание и управление бронированиями
- ✅ Сканирование QR-кодов
- ✅ Система уведомлений
- ✅ Разные роли пользователей (пользователь, администратор)

### Дополнительно для админов:
- ✅ Управление пользователями
- ✅ Управление оборудованием и категориями
- ✅ Управление бронированиями
- ✅ Просмотр логов и отчётов
- ✅ Управление группами и отрядами

---

## 🛠 Решение проблем

### Сервер не запускается

```bash
# Проверьте, что PostgreSQL запущен
psql -U postgres -l

# Проверьте наличие базы данных
psql -U postgres -d kvantoriym -c "SELECT 1"

# Обновите Rust
rustup update
```

### Порт занят

```bash
# Linux/Mac: найдите процесс
lsof -i :8080

# Windows PowerShell
netstat -ano | findstr :8080

# Используйте другой порт в .env
PORT=8081
```

### Ошибка подключения из Android

- Для эмулятора используйте `10.0.2.2` вместо `localhost`
- Для реального устройства используйте IP вашего компьютера
- Убедитесь, что устройство и сервер в одной сети

### Node modules конфликты

```bash
# Очистите кэш и переустановите
rm -rf node_modules package-lock.json
npm install
```

---

## 📚 Дополнительная документация

- [Сервер](invenbase-server/README.md) - REST API документация
- [Быстрый старт сервера](invenbase-server/QUICKSTART.md) - Шаг за шагом
- [Клиент](invenbase-client/README.md) - Веб и Android веб-версия
- [Android нативное](invenbase-android/README.md) - Нативное приложение
- [Инструкции сборки Android](invenbase-android/BUILD_INSTRUCTIONS.md)
- [Firebase настройка](invenbase-android/FIREBASE_SETUP.md) (если используется)
- [Push-уведомления](invenbase-android/PUSH_NOTIFICATIONS_SERVER.md) (если используется)

---

## 🔐 Безопасность

⚠️ **ВАЖНО для продакшена:**

1. **Измените пароль администратора** после первого входа
2. **Установите strong JWT_SECRET** (минимум 32 символа)
3. **Используйте HTTPS** в продакшене
4. **Скрывайте .env файлы** в git через .gitignore
5. **Обновляйте зависимости** регулярно

```bash
# Проверка уязвимостей
npm audit
cargo audit
```

---

## 📝 Разработка

### Структура источников

```
invenbase-server/
├── src/
│   ├── main.rs           # Точка входа
│   ├── app_state.rs      # Состояние приложения
│   ├── handlers/         # Обработчики API
│   ├── models.rs         # Модели данных
│   ├── database.rs       # Работа с БД
│   └── middleware.rs     # Middleware

invenbase-client/
├── web/src/
│   ├── App.jsx           # Главный компонент
│   ├── pages/            # Страницы
│   ├── components/       # Компоненты
│   ├── api/              # API клиент
│   └── context/          # React Context
└── android/              # Android WebView версия

invenbase-android/
└── app/src/main/
    ├── java/com/invenbase/app/
    │   ├── api/          # Retrofit сервисы
    │   ├── fragments/    # Экраны
    │   ├── models/       # Модели данных
    │   └── utils/        # Утилиты
    └── res/              # Ресурсы
```

### Советы по разработке

- Используйте `cargo watch` для автоматической пересборки сервера
- Используйте `npm run dev` для горячей перезагрузки фронтенда
- Читайте логи в консоли для отладки проблем
- Используйте Postman для тестирования API endpoints

---

## 📄 Лицензия

Проект для технопарка "Кванториум"

---

## 🤝 Поддержка

При возникновении проблем:
1. Проверьте логи консоли
2. Убедитесь, что выполнены все требования
3. Прочитайте подробную документацию в папках компонентов
4. Проверьте, что все сервисы запущены

---

## 🎯 Статус разработки

- ✅ Основная функциональность
- ✅ Веб-приложение
- ✅ Android WebView версия
- ✅ Android нативное приложение
- 🔄 Интеграция Firebase
- 🔄 Push-уведомления
- 🔄 Расширенные отчёты

---

**Последнее обновление:** Июнь 2026
