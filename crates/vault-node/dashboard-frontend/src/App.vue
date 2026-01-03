<template>
  <div class="min-h-screen bg-vault-dark text-white">
    <!-- Header -->
    <header class="bg-vault-darker border-b border-vault-accent px-6 py-4">
      <div class="flex items-center justify-between">
        <div class="flex items-center space-x-4">
          <h1 class="text-2xl font-bold text-vault-cyan">SolanaVault</h1>
          <span class="text-gray-400">Node Dashboard</span>
        </div>
        <div class="flex items-center space-x-4">
          <div class="flex items-center space-x-2">
            <span class="w-2 h-2 rounded-full" :class="connectionClass"></span>
            <span class="text-sm text-gray-400">{{ connectionStatus }}</span>
          </div>
          <span v-if="snapshot" class="text-sm text-gray-500">
            v{{ snapshot.node_info.version }}
          </span>
        </div>
      </div>
    </header>

    <!-- Navigation Tabs -->
    <nav class="bg-vault-darker border-b border-vault-accent">
      <div class="flex space-x-1 px-6">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          @click="currentTab = tab.id"
          class="px-4 py-3 text-sm font-medium transition-colors"
          :class="currentTab === tab.id
            ? 'text-vault-cyan border-b-2 border-vault-cyan'
            : 'text-gray-400 hover:text-white'"
        >
          {{ tab.label }}
        </button>
      </div>
    </nav>

    <!-- Main Content -->
    <main class="p-6">
      <div v-if="!snapshot" class="flex items-center justify-center h-64">
        <div class="text-center">
          <div class="animate-spin w-8 h-8 border-2 border-vault-cyan border-t-transparent rounded-full mx-auto mb-4"></div>
          <p class="text-gray-400">Connecting to node...</p>
        </div>
      </div>

      <component
        v-else
        :is="currentComponent"
        :snapshot="snapshot"
        :history="history"
      />
    </main>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, shallowRef } from 'vue'
import Overview from './components/Overview.vue'
import Storage from './components/Storage.vue'
import Network from './components/Network.vue'
import Economics from './components/Economics.vue'

const tabs = [
  { id: 'overview', label: 'Overview' },
  { id: 'storage', label: 'Storage' },
  { id: 'network', label: 'Network' },
  { id: 'economics', label: 'Economics' },
]

const currentTab = ref('overview')
const snapshot = ref(null)
const history = ref(null)
const connected = ref(false)

let ws = null
let reconnectTimeout = null

const components = {
  overview: Overview,
  storage: Storage,
  network: Network,
  economics: Economics,
}

const currentComponent = computed(() => components[currentTab.value])

const connectionStatus = computed(() => connected.value ? 'Connected' : 'Disconnected')
const connectionClass = computed(() => connected.value ? 'bg-vault-green status-pulse' : 'bg-vault-red')

function connect() {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
  const wsUrl = `${protocol}//${window.location.host}/ws`

  ws = new WebSocket(wsUrl)

  ws.onopen = () => {
    connected.value = true
    console.log('WebSocket connected')
  }

  ws.onmessage = (event) => {
    try {
      snapshot.value = JSON.parse(event.data)
    } catch (e) {
      console.error('Failed to parse message:', e)
    }
  }

  ws.onclose = () => {
    connected.value = false
    console.log('WebSocket disconnected, reconnecting...')
    reconnectTimeout = setTimeout(connect, 2000)
  }

  ws.onerror = (error) => {
    console.error('WebSocket error:', error)
  }
}

async function fetchHistory() {
  try {
    const res = await fetch('/api/history')
    history.value = await res.json()
  } catch (e) {
    console.error('Failed to fetch history:', e)
  }
}

onMounted(() => {
  connect()
  fetchHistory()
  // Refresh history every 10 seconds
  const historyInterval = setInterval(fetchHistory, 10000)
  onUnmounted(() => clearInterval(historyInterval))
})

onUnmounted(() => {
  if (ws) ws.close()
  if (reconnectTimeout) clearTimeout(reconnectTimeout)
})
</script>
