<script setup lang="ts">
defineProps<{ show: boolean }>();
const emit = defineEmits<{ 'close': [] }>();

const hosts = ['TS', 'TY'];
const mentors = ['YB', 'AS', 'KY', 'MA'];
const feedbacks = ['RK(友人)', 'RN(友人・9期生)'];
</script>

<template>
  <transition name="modal-fade">
    <div v-if="show" class="modal-overlay" @click.self="emit('close')">
      <div class="modal-card">
        <header class="modal-header">
          <div class="v9-tag">100 program v9</div>
          <h2 class="modal-title">TaskGraph</h2>
          <button class="close-btn" @click="emit('close')" aria-label="Close">&times;</button>
        </header>

        <div class="modal-body">
          <div class="author-section">
            <span class="label">Developed by</span>
            <h3 class="author-name">Suzuha Takaki</h3>
            <div class="author-links">
              <a href="https://github.com/aki91-g" target="_blank" rel="noopener noreferrer" class="link">GitHub Profile →</a>
            </div>
          </div>

          <div class="program-info">
            <p class="intro-text">
              TaskGraphは <strong>100 program 9期 (2026春期)</strong> において製作されました。
            </p>
            <a href="https://100program.jp" target="_blank" rel="noopener noreferrer" class="link">100 program Official Site →</a>
          </div>

          <div class="divider"></div>

          <div class="thanks-container">
            <h3 class="thanks-title">Special Thanks to</h3>
            <div class="thanks-grid">
              <section class="thanks-item">
                <span class="role-label">Program Hosts</span>
                <ul class="name-grid">
                  <li v-for="name in hosts" :key="name" class="name-chip">{{ name }}</li>
                </ul>
              </section>

              <section class="thanks-item">
                <span class="role-label">Special Support</span>
                <ul class="name-grid">
                  <li v-for="name in mentors" :key="name" class="name-chip">{{ name }}</li>
                </ul>
              </section>

              <section class="thanks-item thanks-item-wide">
                <span class="role-label">Feedback from</span>
                <ul class="name-grid feedback-grid">
                  <li v-for="name in feedbacks" :key="name" class="name-chip">{{ name }}</li>
                </ul>
              </section>
            </div>
          </div>
        </div>

        <footer class="modal-footer">
          <p class="copyright">&copy; 2026 TaskGraph Project</p>
        </footer>
      </div>
    </div>
  </transition>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  z-index: 9999;
  background: rgba(15, 23, 42, 0.75);
  backdrop-filter: blur(10px);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1.5rem;
}

/* メインカード */
.modal-card {
  background: #ffffff;
  width: 100%;
  max-width: 720px;
  max-height: min(88vh, 820px);
  border-radius: 2rem;
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
  overflow: hidden;
  border: 1px solid rgba(255, 255, 255, 0.3);
  display: flex;
  flex-direction: column;
}

/* ヘッダー */
.modal-header {
  padding: 2rem 2rem 1rem;
  text-align: center;
  position: relative;
  background: linear-gradient(to bottom, #f8fafc, #ffffff);
  flex-shrink: 0;
}

.modal-title {
  font-size: 2rem;
  font-weight: 900;
  color: #0f172a;
  letter-spacing: -0.05em;
  margin: 0;
}

.v9-tag {
  display: inline-block;
  font-size: 0.75rem;
  font-weight: 800;
  padding: 0.25rem 1rem;
  border-radius: 999px;
  background: linear-gradient(135deg, #ef4444, #a855f7, #3b82f6);
  color: white;
  margin-bottom: 0.75rem;
  box-shadow: 0 4px 12px rgba(168, 85, 247, 0.2);
}

.close-btn {
  position: absolute;
  top: 1.5rem;
  right: 1.5rem;
  font-size: 2rem;
  background: none;
  border: none;
  color: #94a3b8;
  cursor: pointer;
  line-height: 1;
  transition: color 0.2s;
}

.close-btn:hover { color: #64748b; }

/* ボディ */
.modal-body {
  padding: 1rem 2.25rem 2rem;
  text-align: center;
  min-height: 0;
  overflow-y: auto;
}

.author-section {
  background: #f8fafc;
  padding: 1.5rem;
  border-radius: 1.5rem;
  margin-bottom: 1.5rem;
  border: 1px solid #f1f5f9;
}

.author-section .label {
  display: block;
  font-size: 0.7rem;
  text-transform: uppercase;
  letter-spacing: 0.15em;
  color: #64748b;
  margin-bottom: 0.5rem;
}

.author-name {
  font-size: 1.75rem;
  font-weight: 900;
  color: #0f172a;
  margin: 0 0 0.5rem 0;
}

.intro-text {
  font-size: 0.95rem;
  line-height: 1.6;
  color: #475569;
}

.divider {
  height: 1px;
  background: linear-gradient(to right, transparent, #e2e8f0, transparent);
  margin: 2.5rem 0;
}

.thanks-title {
  font-size: 1.5rem;
  font-weight: 950;
  color: #0f172a;
  margin-bottom: 1.5rem;
  letter-spacing: -0.06em;
}

.thanks-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 1rem;
  align-items: stretch;
}

.thanks-item {
  border: 1px solid #e2e8f0;
  border-radius: 1rem;
  padding: 1rem;
  background: linear-gradient(180deg, #ffffff 0%, #f8fafc 100%);
  text-align: center;
}

.thanks-item-wide {
  grid-column: 1 / -1;
}

.role-label {
  display: block;
  font-size: 0.65rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: #94a3b8;
  margin-bottom: 0.5rem;
}

.person-name {
  font-size: 1.15rem;
  font-weight: 800;
  color: #1e293b;
  padding: 0.4rem 1rem;
  margin: 0;
}

.name-grid {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.6rem;
}

.feedback-grid {
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
}

.name-chip {
  border: 1px solid #dbe3ee;
  background: #ffffff;
  border-radius: 0.75rem;
  padding: 0.5rem 0.65rem;
  font-size: 1rem;
  font-weight: 700;
  color: #334155;
}

.link {
  color: #3b82f6;
  text-decoration: none;
  font-size: 0.85rem;
  font-weight: 700;
  transition: opacity 0.2s;
}

.link:hover { opacity: 0.7; }

.modal-footer {
  padding: 1.25rem;
  background: #f8fafc;
  border-top: 1px solid #f1f5f9;
  flex-shrink: 0;
}

.copyright {
  font-size: 0.75rem;
  color: #94a3b8;
  margin: 0;
}

.modal-fade-enter-active, .modal-fade-leave-active {
  transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1);
}
.modal-fade-enter-from, .modal-fade-leave-to {
  opacity: 0;
  transform: scale(0.9) translateY(20px);
}

@media (max-width: 760px) {
  .modal-overlay {
    padding: 0.75rem;
    align-items: flex-end;
  }

  .modal-card {
    max-height: 92vh;
    border-radius: 1.25rem;
  }

  .modal-header {
    padding: 1.25rem 1rem 0.75rem;
  }

  .modal-body {
    padding: 0.75rem 1rem 1.25rem;
  }

  .thanks-grid {
    grid-template-columns: 1fr;
  }

  .thanks-item-wide {
    grid-column: auto;
  }

  .name-grid,
  .feedback-grid {
    grid-template-columns: 1fr;
  }
}
</style>