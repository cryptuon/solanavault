<template>
  <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
    <!-- Connection Stats -->
    <div class="card">
      <h2 class="text-lg font-semibold text-vault-cyan mb-4">Connection Statistics</h2>
      <div class="grid grid-cols-2 gap-6 mb-6">
        <div class="text-center">
          <div class="text-4xl font-bold" :class="peerColor">
            {{ snapshot.network.connected_peers }}
          </div>
          <div class="text-gray-400">Connected Peers</div>
        </div>
        <div class="text-center">
          <div class="text-4xl font-bold text-white">
            {{ snapshot.network.total_peers }}
          </div>
          <div class="text-gray-400">Total Peers</div>
        </div>
      </div>

      <div class="space-y-4">
        <StatRow label="Connection Rate" :value="connectionRate" :color="connectionRateColor" />
        <StatRow label="Average Latency" :value="snapshot.network.average_latency_ms.toFixed(1) + ' ms'" color="yellow" />
        <StatRow label="Bandwidth In" :value="formatBandwidth(snapshot.network.bandwidth_in_bytes)" color="green" />
        <StatRow label="Bandwidth Out" :value="formatBandwidth(snapshot.network.bandwidth_out_bytes)" color="cyan" />
      </div>
    </div>

    <!-- Message Stats -->
    <div class="card">
      <h2 class="text-lg font-semibold text-vault-cyan mb-4">Message Statistics</h2>
      <div class="grid grid-cols-2 gap-6 mb-6">
        <div class="text-center">
          <div class="text-3xl font-bold text-vault-green">
            {{ formatNumber(snapshot.network.messages_sent) }}
          </div>
          <div class="text-gray-400">Messages Sent</div>
        </div>
        <div class="text-center">
          <div class="text-3xl font-bold text-vault-cyan">
            {{ formatNumber(snapshot.network.messages_received) }}
          </div>
          <div class="text-gray-400">Messages Received</div>
        </div>
      </div>

      <div class="space-y-4">
        <StatRow label="Total Messages" :value="formatNumber(totalMessages)" />
        <StatRow
          label="Send/Receive Ratio"
          :value="sendReceiveRatio"
        />
      </div>
    </div>

    <!-- Consensus Stats -->
    <div class="card">
      <h2 class="text-lg font-semibold text-vault-cyan mb-4">Consensus Statistics</h2>
      <div class="text-center mb-6">
        <div class="text-5xl font-bold text-vault-cyan">
          {{ snapshot.consensus.reputation_score.toFixed(2) }}
        </div>
        <div class="text-gray-400 mt-1">Reputation Score</div>
      </div>

      <div class="grid grid-cols-2 gap-4 mb-4">
        <div class="text-center p-4 bg-vault-accent rounded-lg">
          <div class="text-2xl font-bold text-vault-green">{{ snapshot.consensus.proposals_accepted }}</div>
          <div class="text-sm text-gray-400">Accepted</div>
        </div>
        <div class="text-center p-4 bg-vault-accent rounded-lg">
          <div class="text-2xl font-bold text-vault-red">{{ snapshot.consensus.proposals_rejected }}</div>
          <div class="text-sm text-gray-400">Rejected</div>
        </div>
      </div>

      <div class="space-y-4">
        <StatRow label="Active Proposals" :value="snapshot.consensus.active_proposals.toString()" color="yellow" />
        <StatRow label="Votes Cast" :value="formatNumber(snapshot.consensus.votes_cast)" />
      </div>
    </div>

    <!-- Network Health -->
    <div class="card">
      <h2 class="text-lg font-semibold text-vault-cyan mb-4">Network Health</h2>

      <div class="space-y-6">
        <!-- Peer Health Indicator -->
        <div>
          <div class="flex justify-between mb-2">
            <span class="text-gray-400">Peer Connectivity</span>
            <span :class="peerHealthColor">{{ peerHealthStatus }}</span>
          </div>
          <div class="gauge-bar h-3">
            <div
              class="gauge-fill"
              :class="peerHealthGaugeClass"
              :style="{ width: peerHealthPercent + '%' }"
            ></div>
          </div>
        </div>

        <!-- Message Flow -->
        <div>
          <div class="flex justify-between mb-2">
            <span class="text-gray-400">Message Flow</span>
            <span class="text-vault-cyan">{{ messageFlowStatus }}</span>
          </div>
          <div class="flex items-center space-x-2">
            <div class="flex-1 h-2 bg-vault-accent rounded-full overflow-hidden">
              <div class="h-full bg-vault-green" :style="{ width: sentPercent + '%' }"></div>
            </div>
            <span class="text-xs text-gray-500 w-20 text-right">{{ sentPercent.toFixed(0) }}% sent</span>
          </div>
        </div>

        <!-- Consensus Health -->
        <div>
          <div class="flex justify-between mb-2">
            <span class="text-gray-400">Consensus Participation</span>
            <span :class="consensusHealthColor">{{ consensusHealthStatus }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps(['snapshot', 'history'])

// Inline StatRow component
const StatRow = {
  props: ['label', 'value', 'color'],
  template: `
    <div class="flex justify-between items-center py-2 border-b border-vault-accent last:border-0">
      <span class="text-gray-400">{{ label }}</span>
      <span class="font-semibold" :class="colorClass">{{ value }}</span>
    </div>
  `,
  computed: {
    colorClass() {
      const colors = {
        cyan: 'text-vault-cyan',
        green: 'text-vault-green',
        yellow: 'text-vault-yellow',
        red: 'text-vault-red',
      }
      return colors[this.color] || 'text-white'
    }
  }
}

const peerColor = computed(() => {
  return props.snapshot.network.connected_peers > 0 ? 'text-vault-green' : 'text-vault-red'
})

const connectionRate = computed(() => {
  if (!props.snapshot.network.total_peers) return 'N/A'
  const rate = (props.snapshot.network.connected_peers / props.snapshot.network.total_peers) * 100
  return rate.toFixed(1) + '%'
})

const connectionRateColor = computed(() => {
  if (!props.snapshot.network.total_peers) return 'white'
  const rate = props.snapshot.network.connected_peers / props.snapshot.network.total_peers
  if (rate >= 0.7) return 'green'
  if (rate >= 0.3) return 'yellow'
  return 'red'
})

const totalMessages = computed(() => {
  return props.snapshot.network.messages_sent + props.snapshot.network.messages_received
})

const sendReceiveRatio = computed(() => {
  if (!props.snapshot.network.messages_received) return 'N/A'
  const ratio = props.snapshot.network.messages_sent / props.snapshot.network.messages_received
  return ratio.toFixed(2) + ':1'
})

const sentPercent = computed(() => {
  if (!totalMessages.value) return 50
  return (props.snapshot.network.messages_sent / totalMessages.value) * 100
})

const peerHealthPercent = computed(() => {
  if (!props.snapshot.network.total_peers) return 0
  return Math.min((props.snapshot.network.connected_peers / props.snapshot.network.total_peers) * 100, 100)
})

const peerHealthStatus = computed(() => {
  if (peerHealthPercent.value >= 70) return 'Healthy'
  if (peerHealthPercent.value >= 30) return 'Degraded'
  return 'Critical'
})

const peerHealthColor = computed(() => {
  if (peerHealthPercent.value >= 70) return 'text-vault-green'
  if (peerHealthPercent.value >= 30) return 'text-vault-yellow'
  return 'text-vault-red'
})

const peerHealthGaugeClass = computed(() => {
  if (peerHealthPercent.value >= 70) return 'bg-vault-green'
  if (peerHealthPercent.value >= 30) return 'bg-vault-yellow'
  return 'bg-vault-red'
})

const messageFlowStatus = computed(() => {
  if (totalMessages.value > 1000) return 'High'
  if (totalMessages.value > 100) return 'Normal'
  return 'Low'
})

const consensusHealthStatus = computed(() => {
  if (props.snapshot.consensus.reputation_score >= 0.9) return 'Excellent'
  if (props.snapshot.consensus.reputation_score >= 0.7) return 'Good'
  if (props.snapshot.consensus.reputation_score >= 0.5) return 'Fair'
  return 'Poor'
})

const consensusHealthColor = computed(() => {
  if (props.snapshot.consensus.reputation_score >= 0.9) return 'text-vault-green'
  if (props.snapshot.consensus.reputation_score >= 0.7) return 'text-vault-cyan'
  if (props.snapshot.consensus.reputation_score >= 0.5) return 'text-vault-yellow'
  return 'text-vault-red'
})

function formatNumber(num) {
  if (num >= 1e9) return (num / 1e9).toFixed(2) + 'B'
  if (num >= 1e6) return (num / 1e6).toFixed(2) + 'M'
  if (num >= 1e3) return (num / 1e3).toFixed(2) + 'K'
  return num.toString()
}

function formatBandwidth(bytes) {
  if (bytes >= 1e9) return (bytes / 1e9).toFixed(2) + ' GB/s'
  if (bytes >= 1e6) return (bytes / 1e6).toFixed(2) + ' MB/s'
  if (bytes >= 1e3) return (bytes / 1e3).toFixed(2) + ' KB/s'
  return bytes + ' B/s'
}
</script>
