<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';
import { useAuth } from '@/composables/useAuth';

const { login, signUp, continueAsGuest } = useAuth();

const isRegisterMode = ref(false);
const emailInput = ref('');
const usernameInput = ref('');
const passwordInput = ref('');
const isLogging = ref(false);
const error = ref<string | null>(null);
const isOnline = ref(typeof navigator === 'undefined' ? true : navigator.onLine);

const MIN_USERNAME_LEN = 3;

function normalizeUsername(value: string): string {
  return value.trim();
}

function usernameErrorMessage(value: string): string | null {
  const normalized = normalizeUsername(value);
  if (!normalized) return 'Username is required';
  if (normalized.length < MIN_USERNAME_LEN) {
    return `Username must be at least ${MIN_USERNAME_LEN} characters`;
  }
  return null;
}

function clearError(): void {
  error.value = null;
}

function updateOnlineStatus(): void {
  isOnline.value = navigator.onLine;
}

onMounted(() => {
  window.addEventListener('online', updateOnlineStatus);
  window.addEventListener('offline', updateOnlineStatus);
});

onUnmounted(() => {
  window.removeEventListener('online', updateOnlineStatus);
  window.removeEventListener('offline', updateOnlineStatus);
});

async function handleLogin() {
  if (!emailInput.value.trim() || !passwordInput.value) {
    error.value = 'Please enter both email and password';
    return;
  }

  isLogging.value = true;
  error.value = null;

  try {
    await login(emailInput.value.trim(), passwordInput.value);
  } catch (err) {
    error.value = String(err);
  } finally {
    isLogging.value = false;
  }
}

async function handleRegister() {
  if (!emailInput.value.trim() || !passwordInput.value) {
    error.value = 'Please enter both email and password';
    return;
  }

  const usernameError = usernameErrorMessage(usernameInput.value);
  if (usernameError) {
    error.value = usernameError;
    return;
  }

  if (!isOnline.value) {
    error.value = 'Creating an account requires an internet connection to sync your data.';
    return;
  }

  isLogging.value = true;
  error.value = null;

  try {
    await signUp(emailInput.value.trim(), passwordInput.value, normalizeUsername(usernameInput.value));
  } catch (err) {
    error.value = String(err);
  } finally {
    isLogging.value = false;
  }
}

function handleGuestLogin(): void {
  clearError();
  continueAsGuest();
}
</script>

<template>
  <div class="login-container">
    <div class="bg-glow">
      <div class="glow-center" style="top: 60%; left: 40%; background: radial-gradient(circle, rgba(37, 99, 235, 0.2) 0%, transparent 70%);"></div>
    </div>

    <div class="login-card">
      <div class="header-section">
        <h2>{{ isRegisterMode ? '📝 Create Account' : '🔐 Login' }}</h2>
        <p class="description">
          {{ isRegisterMode ? 'Create your account to start syncing tasks.' : 'Enter your email and password' }}
        </p>
      </div>
      
      <form @submit.prevent="isRegisterMode ? handleRegister() : handleLogin()" class="login-form">
        <div class="input-group">
          <label for="email-input">Email</label>
          <input
            id="email-input"
            v-model="emailInput"
            type="email"
            placeholder="name@example.com"
            :disabled="isLogging"
            class="user-input"
            @input="clearError"
          />
        </div>

        <div class="input-group" v-if="isRegisterMode">
          <label for="username-input">Username</label>
          <input
            id="username-input"
            v-model="usernameInput"
            type="text"
            :placeholder="`At least ${MIN_USERNAME_LEN} characters.`"
            :disabled="isLogging"
            class="user-input"
            @input="clearError"
          />
        </div>

        <div class="input-group">
          <div>
          <label for="password-input">Password</label>
          <input
            id="password-input"
            v-model="passwordInput"
            type="password"
            placeholder="••••••••"
            :disabled="isLogging"
            class="user-input"
            @input="clearError"
          />
           </div>
        </div>

        <button
          type="submit"
          :disabled="
            isLogging ||
            !emailInput.trim() ||
            !passwordInput ||
            (isRegisterMode && (!usernameInput.trim() || !!usernameErrorMessage(usernameInput) || !isOnline))
          "
          class="primary-button"
        >
          <span v-if="isLogging">{{ isRegisterMode ? 'Creating account...' : 'Logging in...' }}</span>
          <span v-else>{{ isRegisterMode ? 'Sign Up' : 'Login' }}</span>
        </button>

        <button
          type="button"
          class="guest-button"
          :disabled="isLogging"
          @click="handleGuestLogin"
        >
          Continue as Guest
        </button>

        <Transition name="fade">
          <p v-if="error" class="error-msg" role="alert">{{ error }}</p>
        </Transition>
      </form>

      <div class="switch-mode">
        <span>{{ isRegisterMode ? "Already have an account? " : "Don't have an account? " }}</span>
        <button 
          type="button" 
          class="link-text" 
          @click="isRegisterMode = !isRegisterMode"
          :disabled="isLogging"
        >
          {{ isRegisterMode ? 'Login' : 'Create One' }}
        </button>
      </div>
    </div>
  </div>
</template>


<style scoped>

.login-container {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
  padding: 1.5rem; 
  background: #f8fafc; 
  font-family: 'Inter', -apple-system, sans-serif;
  position: relative;
  overflow: hidden;
}

.login-container::before {
  content: '';
  position: absolute;
  inset: 0;
  background-image: radial-gradient(rgba(0, 0, 0, 0.05) 1px, transparent 1px);
  background-size: 32px 32px;
  z-index: 1;
}

.bg-glow {
  position: absolute;
  inset: 0;
  z-index: 0;
  overflow: hidden;
}

.bg-glow::before,
.bg-glow::after {
  content: '';
  position: absolute;
  width: 600px;
  height: 600px;
  border-radius: 50%;
  filter: blur(100px); 
  opacity: 0.2;
}

.bg-glow::before {
  top: -5%;
  left: -5%;
  background: radial-gradient(circle, #ef4444 0%, transparent 70%);
}

.bg-glow::after {
  bottom: -5%;
  right: -5%;
  background: radial-gradient(circle, #3b82f6 0%, transparent 70%);
}

.glow-center {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 400px;
  height: 400px;
  background: radial-gradient(circle, rgba(168, 85, 247, 0.25) 0%, transparent 70%);
  filter: blur(80px);
  z-index: 0;
}

.login-card {
  width: 100%;
  max-width: 380px;
  padding: 2.5rem 2rem;
  background: rgba(255, 255, 255, 0.7); 
  backdrop-filter: blur(25px);
  -webkit-backdrop-filter: blur(25px);
  border: 1px solid rgba(255, 255, 255, 0.8);
  border-radius: 24px;
  box-shadow: 0 15px 35px rgba(0, 0, 0, 0.05);
  z-index: 10;
  display: flex;
  flex-direction: column;
}

.header-section {
  text-align: center;
  margin-bottom: 2rem;
}

h2 {
  color: #1e293b;
  font-size: 1.5rem;
  font-weight: 800;
  letter-spacing: -0.02em;
  margin-bottom: 0.4rem;
}

.description {
  color: #64748b;
  font-size: 0.875rem;
}

.login-form {
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

.input-group label {
  display: block;
  color: #94a3b8;
  font-size: 0.7rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 0.4rem;
}

.user-input {
  width: 100%;
  background: rgba(255, 255, 255, 0.5);
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  padding: 0.75rem 1rem;
  color: #1e293b;
  font-size: 0.95rem;
  transition: all 0.2s;
}

.user-input:focus {
  outline: none;
  border-color: #a855f7;
  background: white;
  box-shadow: 0 0 0 3px rgba(168, 85, 247, 0.1);
}

.primary-button {
  width: 100%;
  background: linear-gradient(135deg, #ef4444 0%, #a855f7 50%, #3b82f6 100%);
  color: white;
  padding: 0.9rem;
  border-radius: 12px;
  font-weight: 700;
  border: none;
  cursor: pointer;
  margin-top: 0.75rem;
  box-shadow: 0 8px 16px rgba(168, 85, 247, 0.2);
  transition: all 0.3s ease;
}

.primary-button:disabled {
  background: #e2e8f0;
  color: #94a3b8;
  box-shadow: none;
  cursor: not-allowed;
  transform: none;
}

.primary-button:hover:not(:disabled) {
  transform: translateY(-1px);
  filter: brightness(1.05);
}

.guest-button {
  width: 100%;
  background: rgba(255, 255, 255, 0.75);
  border: 1px solid #cbd5e1;
  color: #334155;
  padding: 0.82rem;
  border-radius: 12px;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.2s ease;
}

.guest-button:hover:not(:disabled) {
  background: #ffffff;
  border-color: #94a3b8;
}

.guest-button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.switch-mode {
  text-align: center;
  margin-top: 1.5rem;
  color: #94a3b8;
  font-size: 0.85rem;
}

.link-text {
  color: #a855f7;
  text-decoration: none;
  font-weight: 600;
  cursor: pointer;
  margin-left: 0.25rem;
}

.link-text:hover {
  text-decoration: underline;
}

.error-msg {
  color: #ef4444;
  font-size: 0.8rem;
  margin-top: -0.5rem;
  text-align: center;
}
</style>