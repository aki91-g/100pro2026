<script setup lang="ts">
import { ref } from 'vue';
import { useAuth } from '@/composables/useAuth';

const { login } = useAuth();

const emailInput = ref('');
const passwordInput = ref('');
const isLogging = ref(false);
const error = ref<string | null>(null);

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
</script>

<template>
  <div class="login-container">
    <div class="login-card">
      <h2>🔐 Login</h2>
      <p class="description">Enter your email and password</p>
      
      <form @submit.prevent="handleLogin" class="login-form">
        <div class="input-group">
          <label for="email-input">Email</label>
          <input
            id="email-input"
            v-model="emailInput"
            type="email"
            placeholder="Email"
            :disabled="isLogging"
            class="user-input"
            @input="error = null"
          />
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
            @input="error = null"
          />
        </div>

        <button
          type="submit"
          :disabled="isLogging || !emailInput.trim() || !passwordInput"
          class="login-button"
        >
          <span v-if="isLogging">Logging in...</span>
          <span v-else>Login</span>
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
