<template>
  <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
    <!-- Node Info Card -->
    <div class="card">
      <h2 class="text-lg font-semibold text-vault-cyan mb-4">Node Info</h2>
      <div class="space-y-3">
        <div class="flex justify-between">
          <span class="text-gray-400">Node ID</span>
          <span class="font-mono text-vault-cyan">{{ snapshot.node_info.node_id }}</span>
        </div>
        <div class="flex justify-between">
          <span class="text-gray-400">Address</span>
          <span>{{ snapshot.node_info.address }}</span>
        </div>
        <div class="flex justify-between">
          <span class="text-gray-400">Status</span>
          <span :class="statusClass">{{ snapshot.node_info.status }}</span>
        </div>
        <div class="flex justify-between">
          <span class="text-gray-400">Uptime</span>
          <span class="text-vault-green">{{ formatUptime(snapshot.node_info.uptime_seconds) }}</span>
        </div>
      </div>
    </div>

    <!-- Storage Gauge Card -->
    <div class="card">
      <h2 class="text-lg font-semibold text-vault-cyan mb-4">Storage</h2>
      <div class="mb-4">
        <div class="gauge-bar">
          <div
            class="gauge-fill"
            :class="storageGaugeClass"
            :style="{ width: storagePercent + '%' }"
          ></div>
        </div>
        <div class="flex justify-between mt-2 text-sm">
          <span class="text-gray-400">{{ formatBytes(snapshot.storage.used_capacity) }} used</span>
          <span class="text-gray-400">{{ formatBytes(snapshot.storage.total_capacity) }} total</span>
        </div>
      </div>
      <div class="grid grid-cols-2 gap-4 text-center">
        <div>
          <div class="text-2xl font-bold text-vault-cyan">{{ snapshot.storage.compression_ratio.toFixed(1) }}:1</div>
          <div class="text-sm text-gray-400">Compression</div>
        </div>
        <div>
          <div class="text-2xl font-bold text-vault-green">{{ snapshot.storage.blocks_stored }}</div>
          <div class="text-sm text-gray-400">Blocks Stored</div>
        </div>
      </div>
    </div>

    <!-- Network Stats Card -->
    <div class="card">
      <h2 class="text-lg font-semibold text-vault-cyan mb-4">Network</h2>
      <div class="grid grid-cols-2 gap-4">
        <div class="text-center">
          <div class="text-3xl font-bold" :class="snapshot.network.connected_peers > 0 ? 'text-vault-green' : 'text-vault-red'">
            {{ snapshot.network.connected_peers }}
          </div>
          <div class="text-sm text-gray-400">Connected Peers</div>
        </div>
        <div class="text-center">
          <div class="text-3xl font-bold text-white">
            {{ snapshot.network.total_peers }}
          </div>
          <div class="text-sm text-gray-400">Total Peers</div>
        </div>
        <div class="text-center">
          <div class="text-xl font-bold text-vault-green">{{ snapshot.network.messages_sent }}</div>
          <div class="text-sm text-gray-400">Sent</div>
        </div>
        <div class="text-center">
          <div class="text-xl font-bold text-vault-cyan">{{ snapshot.network.messages_received }}</div>
          <div class="text-sm text-gray-400">Received</div>
        </div>
      </div>
    </div>

    <!-- Economics Summary Card -->
    <div class="card">
      <h2 class="text-lg font-semibold text-vault-cyan mb-4">Economics</h2>
      <div class="space-y-3">
        <div class="flex justify-between">
          <span class="text-gray-400">Own Stake</span>
          <span class="text-vault-yellow font-semibold">{{ formatTokens(snapshot.economics.staking.own_stake) }}</span>
        </div>
        <div class="flex justify-between">
          <span class="text-gray-400">Pending Rewards</span>
          <span class="text-vault-green">{{ formatTokens(snapshot.economics.staking.pending_rewards) }}</span>
        </div>
        <div class="flex justify-between">
          <span class="text-gray-400">Performance Score</span>
          <span :class="performanceClass">{{ snapshot.economics.staking.performance_score.toFixed(2) }}x</span>
        </div>
        <div class="flex justify-between">
          <span class="text-gray-400">Reputation</span>
          <span class="text-vault-cyan">{{ snapshot.consensus.reputation_score.toFixed(2) }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps(['snapshot', 'history'])

const storagePercent = computed(() => {
  if (!props.snapshot?.storage?.total_capacity) return 0
  return (props.snapshot.storage.used_capacity / props.snapshot.storage.total_capacity) * 100
})

const storageGaugeClass = computed(() => {
  const pct = storagePercent.value
  if (pct < 70) return 'bg-vault-cyan'
  if (pct < 90) return 'bg-vault-yellow'
  return 'bg-vault-red'
})

const statusClass = computed(() => {
  const status = props.snapshot?.node_info?.status
  if (status === 'Running') return 'text-vault-green font-semibold'
  if (status === 'Starting' || status === 'Syncing') return 'text-vault-yellow'
  return 'text-vault-red'
})

const performanceClass = computed(() => {
  const score = props.snapshot?.economics?.staking?.performance_score || 0
  if (score >= 1) return 'text-vault-green font-semibold'
  if (score >= 0.5) return 'text-vault-yellow'
  return 'text-vault-red'
})

function formatUptime(seconds) {
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = seconds % 60
  return `${h}h ${m}m ${s}s`
}

function formatBytes(bytes) {
  if (bytes >= 1e12) return (bytes / 1e12).toFixed(2) + ' TB'
  if (bytes >= 1e9) return (bytes / 1e9).toFixed(2) + ' GB'
  if (bytes >= 1e6) return (bytes / 1e6).toFixed(2) + ' MB'
  if (bytes >= 1e3) return (bytes / 1e3).toFixed(2) + ' KB'
  return bytes + ' B'
}

function formatTokens(amount) {
  if (amount >= 1e9) return (amount / 1e9).toFixed(2) + 'B'
  if (amount >= 1e6) return (amount / 1e6).toFixed(2) + 'M'
  if (amount >= 1e3) return (amount / 1e3).toFixed(2) + 'K'
  return amount.toString()
}
</script>
