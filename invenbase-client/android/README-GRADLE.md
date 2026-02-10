# Решение проблем с Gradle

## Проблема: SocketTimeoutException при загрузке Gradle

Если возникают проблемы с загрузкой Gradle дистрибутива, попробуйте следующие решения:

### Решение 1: Ручная загрузка Gradle

1. Скачайте Gradle вручную:
   - Перейдите на https://gradle.org/releases/
   - Скачайте Gradle 8.5 (или версию из gradle-wrapper.properties)
   - Распакуйте в папку `C:\Users\<ВашеИмя>\.gradle\wrapper\dists\gradle-8.5-bin\<hash>\`

2. Или используйте уже установленный Gradle:
   - Если у вас установлен Gradle глобально, Android Studio может использовать его

### Решение 2: Использование зеркала

Если проблемы с доступом к services.gradle.org, можно использовать зеркало:

1. Откройте `gradle/wrapper/gradle-wrapper.properties`
2. Измените URL на зеркало (например, из Китая):
   ```
   distributionUrl=https\://mirrors.cloud.tencent.com/gradle/gradle-8.5-bin.zip
   ```

### Решение 3: Настройка прокси

Если вы за прокси:

1. Создайте файл `gradle.properties` в `C:\Users\<ВашеИмя>\.gradle\`
2. Добавьте:
   ```
   systemProp.http.proxyHost=your.proxy.host
   systemProp.http.proxyPort=8080
   systemProp.https.proxyHost=your.proxy.host
   systemProp.https.proxyPort=8080
   ```

### Решение 4: Использование офлайн режима

Если Gradle уже скачан:

1. В Android Studio: File → Settings → Build, Execution, Deployment → Gradle
2. Выберите "Offline work"

### Решение 5: Очистка кэша

```bash
cd android
./gradlew clean --refresh-dependencies
```

Или в Windows:
```cmd
cd android
gradlew.bat clean --refresh-dependencies
```

## Проверка версии Gradle

Текущая версия указана в `gradle/wrapper/gradle-wrapper.properties`:
- Gradle 8.5 (совместим с Android Gradle Plugin 8.1.0)

## Альтернатива: Использование более старой версии

Если проблемы продолжаются, можно использовать Gradle 7.6.3:

1. Измените `gradle-wrapper.properties`:
   ```
   distributionUrl=https\://services.gradle.org/distributions/gradle-7.6.3-bin.zip
   ```

2. Измените `build.gradle`:
   ```
   id 'com.android.application' version '7.4.2' apply false
   ```

