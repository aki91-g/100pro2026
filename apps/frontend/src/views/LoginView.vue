<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';
import { useAuth } from '@/composables/useAuth';

const { login, signUp, isAuthenticated } = useAuth();

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

function setMode(registerMode: boolean): void {
  isRegisterMode.value = registerMode;
  clearError();
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
    try {
      await login(emailInput.value.trim(), passwordInput.value);
    } catch (loginError) {
      // If signup already established local session/store state, allow redirect via auth gate.
      if (!isAuthenticated.value) {
        throw loginError;
      }
    }
  } catch (err) {
    error.value = String(err);
  } finally {
    isLogging.value = false;
  }
}
</script>

<template>
  <div class="login-container">
    <div class="login-card">
      <h2>{{ isRegisterMode ? '📝 Create Account' : '🔐 Login' }}</h2>
      <p class="description">
        {{ isRegisterMode ? 'Create your account to start syncing tasks.' : 'Enter your email and password' }}
      </p>

      <div class="mode-toggle" role="tablist" aria-label="Authentication mode">
        <button
          type="button"
          :class="['mode-toggle-btn', !isRegisterMode ? 'active' : '']"
          @click="setMode(false)"
          :disabled="isLogging"
        >
          Login
        </button>
        <button
          type="button"
          :class="['mode-toggle-btn', isRegisterMode ? 'active' : '']"
          @click="setMode(true)"
          :disabled="isLogging"
        >
          Sign Up
        </button>
      </div>
      
      <form @submit.prevent="isRegisterMode ? handleRegister() : handleLogin()" class="login-form">
        <div class="input-group">
          <label for="email-input">Email</label>
          <input
            id="email-input"
            v-model="emailInput"
            type="email"
            placeholder="Email"
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
            placeholder="Username"
            :disabled="isLogging"
            class="user-input"
            @input="clearError"
          />
          <p class="hint">At least {{ MIN_USERNAME_LEN }} characters.</p>
        </div>

        <div class="input-group">
          <label for="password-input">Password</label>
          <input
            id="password-input"
            v-model="passwordInput"
            type="password"
            placeholder="Password"
            :disabled="isLogging"
            class="user-input"
            @input="clearError"
          />
        </div>

        <p v-if="isRegisterMode" class="online-disclaimer" role="status" aria-live="polite">
          An active internet connection is required to create a new account.
        </p>

        <button
          type="submit"
          :disabled="
            isLogging ||
            !emailInput.trim() ||
            !passwordInput ||
            (isRegisterMode && (!usernameInput.trim() || !!usernameErrorMessage(usernameInput) || !isOnline))
          "
          class="login-button"
        >
          <span v-if="isLogging">{{ isRegisterMode ? 'Creating account...' : 'Logging in...' }}</span>
          <span v-else>{{ isRegisterMode ? 'Sign Up' : 'Login' }}</span>
        </button>

        <Transition name="fade">
          <p v-if="error" class="error-msg" role="alert" aria-live="assertive">{{ error }}</p>
        </Transition>
      </form>
    </div>
  </div>
</template>

<style scoped>
.login-container {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
  padding: 2rem;
}

.login-card {
  background: #f8f9fa;
  border-radius: 12px;
  padding: 2rem;
  max-width: 400px;
  width: 100%;
  border: 1px solid #eee;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
}

h2 {
  margin: 0 0 0.5rem 0;
  text-align: center;
  color: #2c3e50;
}

.description {
  text-align: center;
  color: #666;
  font-size: 0.9rem;
  margin-bottom: 1.5rem;
}

.mode-toggle {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 1rem;
}

.mode-toggle-btn {
  flex: 1;
  padding: 0.55rem 0.75rem;
  border: 1px solid #d1d5db;
  border-radius: 8px;
  background: #ffffff;
  color: #374151;
  cursor: pointer;
  font-weight: 600;
  transition: all 0.2s;
}

.mode-toggle-btn.active {
  background: #42b883;
  color: white;
  border-color: #42b883;
}

.mode-toggle-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.login-form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.input-group {
  display: flex;
  flex-direction: column;
}

label {
  margin-bottom: 0.5rem;
  font-size: 0.9rem;
  font-weight: 500;
  color: #2c3e50;
}

.user-input {
  padding: 0.75rem;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 1rem;
  transition: border-color 0.2s;
}

.user-input:focus {
  outline: none;
  border-color: #42b883;
}

.user-input:disabled {
  background: #f5f5f5;
  cursor: not-allowed;
}

.login-button {
  padding: 0.75rem 1.5rem;
  background: #42b883;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 1rem;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.2s;
}

.login-button:hover:not(:disabled) {
  background: #35a372;
}

.login-button:disabled {
  background: #94a3b8;
  cursor: not-allowed;
}

.error-msg {
  color: #e53935;
  font-size: 0.85rem;
  margin: 0;
  padding: 0.5rem;
  background: #ffebee;
  border-radius: 4px;
  text-align: center;
}

.hint {
  margin: 0.4rem 0 0;
  font-size: 0.75rem;
  color: #64748b;
}

.online-disclaimer {
  margin: 0;
  padding: 0.65rem 0.75rem;
  background: #fef3c7;
  border: 1px solid #fcd34d;
  border-radius: 6px;
  font-size: 0.8rem;
  color: #92400e;
}

.info-box {
  margin-top: 1.5rem;
  padding: 1rem;
  background: #e3f2fd;
  border-radius: 6px;
  font-size: 0.85rem;
  color: #1976d2;
}

.info-box p {
  margin: 0;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
