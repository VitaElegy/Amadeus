<script setup>
import { ref, onMounted } from 'vue';
import { DesktopIcon, ServerIcon, CodeIcon, PlayCircleIcon } from 'tdesign-icons-vue-next';

const status = ref('Checking connection...');
const consoleOutput = ref([]);
const command = ref('');
const loading = ref(false);

const checkStatus = async () => {
  try {
    const res = await fetch('http://localhost:8080/api/v1/core/status');
    const data = await res.json();
    status.value = data.status;
    log(`System Check: ${data.status}`);
  } catch (e) {
    status.value = 'Offline (Is Spring Boot running?)';
    log('Error connecting to backend.');
  }
};

const sendCommand = async () => {
  if (!command.value) return;
  loading.value = true;
  log(`> ${command.value}`);
  
  try {
    const res = await fetch('http://localhost:8080/api/v1/core/command', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ command: command.value })
    });
    const data = await res.json();
    log(`[CORE]: ${data.core_response}`);
  } catch (e) {
    log(`[ERROR]: Failed to execute command.`);
  } finally {
    command.value = '';
    loading.value = false;
  }
};

const log = (msg) => {
  const time = new Date().toLocaleTimeString();
  consoleOutput.value.push(`[${time}] ${msg}`);
};

onMounted(() => {
  checkStatus();
});
</script>

<template>
  <div class="space-y-6">
    <!-- Header -->
    <header class="flex justify-between items-center mb-8">
      <div>
        <h1 class="text-2xl font-bold text-white mb-1">System Dashboard</h1>
        <p class="text-gray-400 text-sm">Real-time Core Monitoring</p>
      </div>
      <t-tag theme="success" variant="dark" v-if="status.includes('ONLINE')">
        {{ status }}
      </t-tag>
      <t-tag theme="danger" variant="dark" v-else>
        {{ status }}
      </t-tag>
    </header>

    <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
      <!-- Control Panel -->
      <div class="md:col-span-2 space-y-6">
        <div class="bg-github-card border border-github-border rounded-md p-6">
          <h2 class="text-lg font-bold text-white mb-4 flex items-center gap-2">
            <CodeIcon /> Command Dispatcher
          </h2>
          <div class="flex gap-2">
            <t-input 
              v-model="command" 
              placeholder="Enter system command..." 
              class="w-full bg-github-bg"
              @enter="sendCommand"
            />
            <t-button theme="primary" @click="sendCommand" :loading="loading">
              <template #icon><PlayCircleIcon /></template>
              Execute
            </t-button>
          </div>
        </div>

        <div class="bg-github-card border border-github-border rounded-md p-6 h-96 overflow-hidden flex flex-col">
           <h2 class="text-lg font-bold text-white mb-4 flex items-center gap-2">
            <DesktopIcon /> System Log
          </h2>
          <div class="flex-1 overflow-y-auto bg-github-bg p-4 rounded border border-github-border font-mono text-sm">
            <div v-for="(log, i) in consoleOutput" :key="i" class="mb-1">
              <span class="text-green-500">$</span> {{ log }}
            </div>
          </div>
        </div>
      </div>

      <!-- Stats Sidebar -->
      <div class="space-y-6">
        <div class="bg-github-card border border-github-border rounded-md p-6">
          <h3 class="text-sm font-bold text-gray-400 uppercase mb-4">System Metrics</h3>
          <div class="space-y-4">
             <div>
              <div class="flex justify-between mb-1">
                <span>CPU Load</span>
                <span>12%</span>
              </div>
              <t-progress :percentage="12" theme="success" />
             </div>
             <div>
              <div class="flex justify-between mb-1">
                <span>Memory</span>
                <span>45%</span>
              </div>
              <t-progress :percentage="45" theme="warning" />
             </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

