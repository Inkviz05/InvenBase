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

const getDefaultApiUrl = () => {
  if (isAndroidWebView) {
    // В Android эмуляторе используем 10.0.2.2 для доступа к localhost хоста
    // Для реального устройства нужно будет использовать IP адрес компьютера
    const apiUrl = 'http://10.0.2.2:8080/api';
    console.log('Using Android WebView API URL:', apiUrl);
    return apiUrl;
  }
  const apiUrl = 'http://localhost:8080/api';
  console.log('Using default API URL:', apiUrl);
  return apiUrl;
};

// Функция для получения API URL (проверяет динамически)
export const getApiBaseUrl = () => {
  // Сначала проверяем, если Android инжектировал URL
  if (typeof window !== 'undefined' && window.ANDROID_API_URL) {
    console.log('Using Android injected API URL:', window.ANDROID_API_URL);
    return window.ANDROID_API_URL;
  }
  
  // Затем проверяем переменную окружения
  if (import.meta.env.VITE_API_URL) {
    console.log('Using environment API URL:', import.meta.env.VITE_API_URL);
    return import.meta.env.VITE_API_URL;
  }
  
  // Иначе используем функцию определения
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

