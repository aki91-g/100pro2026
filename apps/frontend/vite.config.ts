import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { fileURLToPath, URL } from 'node:url'

export default defineConfig({
  plugins: [vue()],
  envDir: '../../',
  resolve: {
    alias: {
      // This maps '@' to the 'src' directory relative to this config file
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  },
  server: {
    host: '0.0.0.0',
    port: 5173,
    strictPort: true,
    watch: {
      // Necessary for HMR to work inside DevContainers
      usePolling: true
    }
  }
})