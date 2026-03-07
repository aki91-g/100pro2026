import { createApp } from 'vue'
import { createPinia } from 'pinia'
import './style.css'
import App from './App.vue'
import { honoClient } from './api/honoClient'
import { useUserStore } from './stores/user'

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)

// Token lookup is deferred until request time to avoid early-store access.
honoClient.setTokenGetter(() => useUserStore(pinia).accessToken ?? null)

app.mount('#app')
