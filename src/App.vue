<template>
  <div class="app-shell">
    <header class="hero">
      <div>
        <p class="eyebrow">Laravel REPL for local and Docker workflows</p>
        <h1>TinkerLab</h1>
        <p class="subtitle">
          Run Laravel Tinker snippets against a local project or a Docker container with the same polished workflow.
        </p>
      </div>
      <div class="hero-actions">
        <button class="secondary" @click="clearEditor">Clear</button>
        <button class="primary" :disabled="isRunning || !canRun" @click="runTinkerCode">
          {{ isRunning ? 'Running…' : 'Run snippet' }}
        </button>
      </div>
    </header>

    <section class="toolbar card">
      <div class="mode-switcher">
        <button :class="['mode-button', connectionMode === 'local' ? 'active' : '']" @click="connectionMode = 'local'">
          Local Laravel folder
        </button>
        <button :class="['mode-button', connectionMode === 'docker' ? 'active' : '']" @click="connectionMode = 'docker'">
          Docker container
        </button>
      </div>

      <div v-if="connectionMode === 'local'" class="field-group">
        <label class="field">
          <span>Laravel project folder</span>
          <div class="inline-row">
            <input v-model="path" placeholder="/path/to/laravel/project" />
            <button class="secondary" @click="pickFolder">Select folder</button>
          </div>
        </label>
      </div>

      <div v-else class="field-group">
        <label class="field">
          <span>Docker container</span>
          <div class="inline-row">
            <input v-model="dockerContainer" placeholder="laravel-app" />
            <button class="secondary" @click="loadContainers">Refresh</button>
          </div>
          <div class="chip-row">
            <button v-for="container in containers" :key="container" class="chip" @click="dockerContainer = container">
              {{ container }}
            </button>
          </div>
        </label>
        <label class="field">
          <span>Project path inside container</span>
          <input v-model="dockerPath" placeholder="/var/www/html" />
        </label>
      </div>

      <p class="hint">The app validates the Laravel project before running Tinker.</p>
    </section>

    <section class="content-grid">
      <div class="card editor-panel">
        <div class="panel-header">
          <h2>Snippet editor</h2>
          <div class="snippet-buttons">
            <button v-for="snippet in snippets" :key="snippet.label" class="chip" @click="applySnippet(snippet.code)">
              {{ snippet.label }}
            </button>
          </div>
        </div>

        <textarea v-model="phpCode" placeholder="Example: dump(app()->version());"></textarea>

        <div class="panel-footer">
          <p v-if="lastExecutedAt">Last run: {{ lastExecutedAt }}</p>
          <div class="footer-actions">
            <button class="secondary" @click="clearEditor">Reset</button>
            <button class="primary" @click="runTinkerCode">Execute</button>
          </div>
        </div>
      </div>

      <div class="card output-panel">
        <div class="panel-header">
          <h2>Output</h2>
          <button class="ghost" @click="copyOutput">Copy</button>
        </div>

        <div class="output-area">
          <div v-if="statusMessage" class="message info">{{ statusMessage }}</div>
          <div v-else-if="error" class="message error">{{ error }}</div>
          <pre v-else-if="result">{{ result }}</pre>
          <div v-else class="placeholder">Your Tinker output will appear here.</div>
        </div>
      </div>
    </section>

    <section class="card history-panel">
      <h3>Recent snippets</h3>
      <div class="history-list">
        <button v-for="item in history" :key="item" class="history-item" @click="loadHistorySnippet(item)">
          <span>{{ item }}</span>
        </button>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const phpCode = ref('');
const path = ref('');
const result = ref('');
const error = ref('');
const statusMessage = ref('');
const isRunning = ref(false);
const lastExecutedAt = ref('');
const history = ref<string[]>([]);
const connectionMode = ref<'local' | 'docker'>('local');
const dockerContainer = ref('');
const dockerPath = ref('/var/www/html');
const containers = ref<string[]>([]);

const snippets: Array<{ label: string; code: string }> = [
  { label: 'Version', code: 'dump(app()->version());' },
  { label: 'User', code: 'dump(App\\Models\\User::first());' },
  { label: 'Config', code: 'dump(config(\'app.name\'));' },
  { label: 'Collection', code: 'dump(collect([1, 2, 3])->map(fn ($value) => $value * 2));' }
];

const canRun = computed(() => {
  if (connectionMode.value === 'local') {
    return Boolean(path.value.trim() && phpCode.value.trim());
  }

  return Boolean(dockerContainer.value.trim() && dockerPath.value.trim() && phpCode.value.trim());
});

onMounted(() => {
  const storedPath = localStorage.getItem('tinkerlab:path');
  const storedCode = localStorage.getItem('tinkerlab:code');
  const storedHistory = localStorage.getItem('tinkerlab:history');
  const storedMode = localStorage.getItem('tinkerlab:mode');
  const storedContainer = localStorage.getItem('tinkerlab:container');
  const storedContainerPath = localStorage.getItem('tinkerlab:containerPath');

  if (storedPath) {
    path.value = storedPath;
  }

  if (storedCode) {
    phpCode.value = storedCode;
  }

  if (storedHistory) {
    try {
      history.value = JSON.parse(storedHistory) as string[];
    } catch {
      history.value = [];
    }
  }

  if (storedMode === 'docker') {
    connectionMode.value = 'docker';
  }

  if (storedContainer) {
    dockerContainer.value = storedContainer;
  }

  if (storedContainerPath) {
    dockerPath.value = storedContainerPath;
  }

  loadContainers().catch(() => undefined);
});

watch(path, (value) => localStorage.setItem('tinkerlab:path', value));
watch(phpCode, (value) => localStorage.setItem('tinkerlab:code', value));
watch(history, (value) => localStorage.setItem('tinkerlab:history', JSON.stringify(value)));
watch(connectionMode, (value) => localStorage.setItem('tinkerlab:mode', value));
watch(dockerContainer, (value) => localStorage.setItem('tinkerlab:container', value));
watch(dockerPath, (value) => localStorage.setItem('tinkerlab:containerPath', value));

const applySnippet = (code: string) => {
  phpCode.value = code;
};

const clearEditor = () => {
  phpCode.value = '';
  result.value = '';
  error.value = '';
  statusMessage.value = '';
};

const addToHistory = (code: string) => {
  const trimmed = code.trim();
  if (!trimmed) {
    return;
  }

  history.value = [trimmed, ...history.value.filter((item) => item !== trimmed)].slice(0, 8);
};

const loadHistorySnippet = (code: string) => {
  phpCode.value = code;
};

const copyOutput = async () => {
  const text = error.value || result.value || statusMessage.value;

  if (!text) {
    return;
  }

  try {
    await navigator.clipboard.writeText(text);
    statusMessage.value = 'Output copied to clipboard.';
  } catch {
    error.value = 'Clipboard access is not available in this environment.';
  }
};

const pickFolder = async () => {
  try {
    const selected = await invoke<string>('pick_laravel_directory');
    path.value = selected;
    statusMessage.value = 'Laravel folder selected.';
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err);
  }
};

const loadContainers = async () => {
  try {
    const list = await invoke<string[]>('list_docker_containers');
    containers.value = list;
    statusMessage.value = containers.value.length ? 'Docker containers refreshed.' : 'No Docker containers were detected.';
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    error.value = message;
    containers.value = [];
  }
};

const runTinkerCode = async () => {
  if (!canRun.value) {
    error.value = 'Provide the required project connection details and snippet before running.';
    return;
  }

  isRunning.value = true;
  result.value = '';
  error.value = '';
  statusMessage.value = 'Executing snippet…';

  try {
    const output = await invoke<string>('run_php_in_tinker', {
      path: connectionMode.value === 'local' ? path.value.trim() : '',
      code: phpCode.value.trim(),
      mode: connectionMode.value,
      containerName: dockerContainer.value.trim() || undefined,
      containerPath: dockerPath.value.trim() || undefined
    });

    result.value = output || 'Command completed successfully.';
    lastExecutedAt.value = new Date().toLocaleString();
    addToHistory(phpCode.value);
    statusMessage.value = '';
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    result.value = '';
    error.value = message;
    statusMessage.value = '';
  } finally {
    isRunning.value = false;
  }
};
</script>

<style>
:root {
  color-scheme: dark;
  font-family: Inter, system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  background: #07111f;
  color: #f5f7fb;
}

* {
  box-sizing: border-box;
}

body {
  margin: 0;
  background: linear-gradient(135deg, #07111f 0%, #0d1b2d 100%);
  color: #f5f7fb;
}

button,
input,
textarea {
  font: inherit;
}

button {
  border: 0;
  cursor: pointer;
  transition: transform 0.2s ease, background 0.2s ease;
}

button:hover {
  transform: translateY(-1px);
}

button:disabled {
  opacity: 0.55;
  cursor: not-allowed;
  transform: none;
}

#app {
  min-height: 100vh;
}

.app-shell {
  max-width: 1280px;
  margin: 0 auto;
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.hero {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
  padding: 24px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 18px;
  background: rgba(8, 15, 29, 0.82);
  backdrop-filter: blur(12px);
}

.eyebrow {
  margin: 0 0 6px;
  text-transform: uppercase;
  letter-spacing: 0.24em;
  color: #86b1ff;
  font-size: 0.78rem;
}

h1,
h2,
h3,
p {
  margin: 0;
}

.subtitle {
  margin-top: 8px;
  color: #9db0c8;
  max-width: 640px;
}

.hero-actions,
.footer-actions {
  display: flex;
  gap: 10px;
}

.primary,
.secondary,
.ghost,
.chip,
.history-item,
.mode-button {
  padding: 10px 14px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.06);
  color: #f5f7fb;
}

.primary {
  background: linear-gradient(135deg, #3b82f6, #2563eb);
}

.secondary,
.ghost,
.mode-button {
  border: 1px solid rgba(255, 255, 255, 0.14);
}

.mode-button.active {
  background: linear-gradient(135deg, #4f46e5, #7c3aed);
}

.card {
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 18px;
  background: rgba(8, 15, 29, 0.82);
  padding: 18px;
}

.toolbar {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.mode-switcher {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

.field-group {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field span {
  color: #86b1ff;
  font-size: 0.9rem;
}

.inline-row {
  display: flex;
  gap: 10px;
}

.inline-row input,
.field input,
textarea {
  width: 100%;
  border: 1px solid rgba(255, 255, 255, 0.14);
  background: rgba(255, 255, 255, 0.04);
  color: #f5f7fb;
  border-radius: 12px;
  padding: 12px 14px;
}

.inline-row input {
  min-height: 46px;
}

.chip-row {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.hint {
  color: #7d8ea6;
  font-size: 0.9rem;
}

.content-grid {
  display: grid;
  grid-template-columns: 1.1fr 0.9fr;
  gap: 18px;
}

.editor-panel,
.output-panel {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.snippet-buttons {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

textarea {
  min-height: 280px;
  resize: vertical;
  line-height: 1.5;
}

.output-area {
  min-height: 280px;
  border-radius: 14px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  padding: 14px;
  background: rgba(0, 0, 0, 0.2);
  overflow: auto;
}

pre {
  white-space: pre-wrap;
  word-break: break-word;
  margin: 0;
  color: #dce7ff;
}

.placeholder {
  color: #7d8ea6;
}

.message {
  font-weight: 600;
}

.info {
  color: #93c5fd;
}

.error {
  color: #ff8f8f;
}

.history-panel {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.history-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.history-item {
  text-align: left;
  max-width: 100%;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

@media (max-width: 900px) {
  .content-grid {
    grid-template-columns: 1fr;
  }

  .hero {
    flex-direction: column;
    align-items: flex-start;
  }

  .inline-row {
    flex-direction: column;
  }
}
</style>
