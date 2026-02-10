#!/bin/bash

# Скрипт для сборки веб-приложения и копирования в Android проект

echo "Сборка веб-приложения..."
cd web
npm install
npm run build

echo "Копирование файлов в Android проект..."
cd ..
mkdir -p android/app/src/main/assets
cp -r web/dist/* android/app/src/main/assets/

echo "Исправление HTML для Android WebView..."
sed -i.bak 's/type="module" crossorigin//g' android/app/src/main/assets/index.html
sed -i.bak 's/crossorigin//g' android/app/src/main/assets/index.html
sed -i.bak 's|<base href="./" />|<base href="file:///android_asset/" />|g' android/app/src/main/assets/index.html
# Инжектируем скрипт для установки Android переменных перед другими скриптами
sed -i.bak 's|<head>|<head>\n    <script>window.ANDROID_WEBVIEW = true; window.ANDROID_API_URL = "http://10.0.2.2:8080/api"; console.log("Android WebView variables set:", window.ANDROID_API_URL);</script>|g' android/app/src/main/assets/index.html
rm -f android/app/src/main/assets/index.html.bak

echo "Готово! Теперь откройте Android Studio и соберите APK."

