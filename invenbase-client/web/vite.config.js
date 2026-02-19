import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// Порт бэкенда (должен совпадать с PORT в invenbase-server/.env)
const API_PORT = process.env.VITE_API_PORT || 8080

export default defineConfig({
  plugins: [react()],
  base: './',
  build: {
    outDir: 'dist',
    assetsDir: 'assets',
  },
  server: {
    port: 3000,
    host: true, // доступ с других устройств по IP
    proxy: {
      // Все запросы на /api идут на бэкенд — не нужен фаервол и один порт для клиента
      '/api': {
        target: `http://127.0.0.1:${API_PORT}`,
        changeOrigin: true,
      },
    },
  },
})

