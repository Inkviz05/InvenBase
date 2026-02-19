// Конфигурация API
// В Android эмуляторе localhost не работает, нужно использовать 10.0.2.2
// Проверяем, находимся ли мы в Android WebView
const detectAndroidWebView = () => {
  if (typeof window === 'undefined') return false;
  
  // Проверяем user agent
  const userAgent = window.navigator.userAgent || '';
  const isAndroid = userAgent.includes('Android');
  const isWebView = userAgent.includes('wv') || userAgent.includes('WebView');
  
  // Проверяем протокол (file:// для локальных файлов)
  const isFileProtocol = window.location.protocol === 'file:';
  
  // Проверяем, если Android инжектировал переменную
  const androidInjected = window.ANDROID_WEBVIEW === true;
  
  const result = isAndroid && (isWebView || isFileProtocol || androidInjected);
  
  console.log('Android WebView Detection:', {
    userAgent,
    isAndroid,
    isWebView,
    isFileProtocol,
    androidInjected,
    result
  });
  
  return result;
};

const isAndroidWebView = detectAndroidWebView();

// Порт API: из .env (VITE_API_PORT) или 8080. Если сервер на 8000 — задайте VITE_API_PORT=8000 в web/.env
const API_PORT = parseInt(import.meta.env.VITE_API_PORT, 10) || 8080;

const getDefaultApiUrl = () => {
  if (isAndroidWebView) {
    const apiUrl = `http://10.0.2.2:${API_PORT}/api`;
    console.log('Using Android WebView API URL:', apiUrl);
    return apiUrl;
  }
  // В браузере: тот же хост, что и страница (работает с других устройств по IP)
  if (typeof window !== 'undefined' && window.location?.hostname) {
    const host = window.location.hostname;
    const apiUrl = `http://${host}:${API_PORT}/api`;
    console.log('Using dynamic API URL (same host):', apiUrl);
    return apiUrl;
  }
  return `http://localhost:${API_PORT}/api`;
};

// Функция для получения API URL (проверяет динамически)
export const getApiBaseUrl = () => {
  if (typeof window !== 'undefined' && window.ANDROID_API_URL) {
    return window.ANDROID_API_URL;
  }
  // В режиме разработки: относительный /api — Vite проксирует на бэкенд. Работает с любого устройства.
  if (import.meta.env.DEV) {
    console.log('Using dev proxy: /api (Vite proxies to backend)');
    return '/api';
  }
  // Продакшен или явный URL из .env
  if (import.meta.env.VITE_API_URL) {
    const url = import.meta.env.VITE_API_URL;
    if (typeof window !== 'undefined' && window.location?.hostname && (url.includes('localhost') || url.includes('127.0.0.1'))) {
      const host = window.location.hostname;
      return url.replace(/^https?:\/\/[^/]+/, `http://${host}:${API_PORT}`);
    }
    return url;
  }
  return getDefaultApiUrl();
};

// Экспортируем как константу для обратной совместимости, но она будет вычисляться динамически
export const API_BASE_URL = getApiBaseUrl();

console.log('Initial API_BASE_URL:', API_BASE_URL);

// Роли пользователей
export const ROLES = {
  ADMIN: 'admin',
  RESPONSIBLE: 'responsible',
  USER: 'user'
};

// Статусы бронирования
export const BOOKING_STATUS = {
  PENDING: 'pending',
  APPROVED: 'approved',
  REJECTED: 'rejected',
  CANCELLED: 'cancelled'
};

// Типы разрешений
export const PERMISSION_TYPE = {
  INTERNAL: 'internal',
  EXTERNAL: 'external'
};

