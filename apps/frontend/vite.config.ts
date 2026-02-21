import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue()],
  server: {
    host: '0.0.0.0', // コンテナ外からのアクセスを許可
    port: 5173,
    strictPort: true,
  }
})
