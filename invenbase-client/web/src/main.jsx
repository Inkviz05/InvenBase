import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import './index.css'

// Функция для безопасной инициализации React
function initApp() {
  const rootElement = document.getElementById('root');
  
  if (!rootElement) {
    console.error('Root element not found! Retrying...');
    // Попробуем еще раз через небольшую задержку
    setTimeout(initApp, 100);
    return;
  }

  // Проверяем, что элемент является валидным DOM-элементом
  if (!(rootElement instanceof HTMLElement)) {
    console.error('Root element is not a valid HTMLElement!');
    return;
  }

  try {
    const root = ReactDOM.createRoot(rootElement);
    root.render(
      <React.StrictMode>
        <App />
      </React.StrictMode>
    );
    console.log('React app initialized successfully');
  } catch (error) {
    console.error('Error initializing React app:', error);
    rootElement.innerHTML = '<div style="padding: 20px; text-align: center; font-family: Arial, sans-serif;"><h1>Ошибка загрузки приложения</h1><p>Пожалуйста, перезагрузите страницу</p><p style="color: #666; font-size: 12px;">' + error.message + '</p></div>';
  }
}

// Ждем полной загрузки DOM
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', initApp);
} else {
  // DOM уже загружен, но дадим небольшую задержку для WebView
  setTimeout(initApp, 50);
}

