import axios from 'axios';
import { getApiBaseUrl } from '../config';

// Создаем axios instance с динамическим baseURL
const apiClient = axios.create({
  baseURL: getApiBaseUrl(),
  headers: {
    'Content-Type': 'application/json',
  },
});

// Обновляем baseURL перед каждым запросом (на случай, если переменные инжектировались позже)
apiClient.interceptors.request.use(
  (config) => {
    // Динамически получаем актуальный API URL
    const currentApiUrl = getApiBaseUrl();
    console.log('API Request:', {
      method: config.method,
      url: config.url,
      baseURL: config.baseURL,
      currentApiUrl: currentApiUrl,
      fullURL: currentApiUrl + config.url,
      data: config.data,
      headers: config.headers
    });
    if (config.baseURL !== currentApiUrl) {
      console.log('Updating API baseURL from', config.baseURL, 'to', currentApiUrl);
      config.baseURL = currentApiUrl;
    }
    return config;
  },
  (error) => {
    console.error('Request interceptor error:', error);
    return Promise.reject(error);
  }
);

// Добавляем токен к каждому запросу (кроме запросов авторизации)
apiClient.interceptors.request.use(
  (config) => {
    // Не добавляем токен к запросам авторизации
    if (config.url && !config.url.includes('/auth/login')) {
      const token = localStorage.getItem('token');
      if (token) {
        config.headers.Authorization = `Bearer ${token}`;
      }
    }
    return config;
  },
  (error) => {
    return Promise.reject(error);
  }
);

// Обработка ошибок
apiClient.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      localStorage.removeItem('token');
      localStorage.removeItem('user');
      // Используем hash для HashRouter
      if (window.location.hash !== '#/login') {
        window.location.hash = '#/login';
      }
    }
    return Promise.reject(error);
  }
);

export default apiClient;

