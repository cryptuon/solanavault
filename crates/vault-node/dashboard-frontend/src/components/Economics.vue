<template>
  <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
    <!-- Staking Stats -->
    <div class="card">
      <h2 class="text-lg font-semibold text-vault-cyan mb-4">Staking</h2>

      <div class="text-center mb-6">
        <div class="text-4xl font-bold text-vault-yellow">
          {{ formatTokens(snapshot.economics.staking.own_stake) }}
        </div>
        <div class="text-gray-400 mt-1">Your Stake</div>
      </div>

      <div class="space-y-4">
        <StatRow
          label="Network Total"
          :value="formatTokens(snapshot.economics.staking.total_staked)"
        />
        <StatRow
          label="Your Share"
          :value="stakeShare"
          color="cyan"
        />
        <StatRow
          label="Pending Rewards"
          :value="formatTokens(snapshot.economics.staking.pending_rewards)"
          color="green"
        />
        <StatRow
          label="Performance Score"
          :value="snapshot.economics.staking.performance_score.toFixed(2) + 'x'"
          :color="performanceColor"
        />
        <StatRow
          label="Base APY"
          :value="(snapshot.economics.staking.base_apy * 100).toFixed(1) + '%'"
        />
        <StatRow
          label="Effective APY"
          :value="effectiveApy"
          color="green"
        />
      </div>
    </div>

    <!-- Rewards Stats -->
    <div class="card">
      <h2 class="text-lg font-semibold text-vault-cyan mb-4">Rewards</h2>

      <div class="text-center mb-6">
        <div class="text-4xl font-bold text-vault-green">
          {{ formatTokens(snapshot.economics.rewards.total_earned) }}
        </div>
        <div class="text-gray-400 mt-1">Total Earned</div>
      </div>

      <div class="space-y-4">
        <StatRow
          label="This Epoch"
          :value="formatTokens(snapshot.economics.rewards.distributed_this_epoch)"
          color="cyan"
        />
        <StatRow
          label="Epochs Completed"
          :value="snapshot.economics.rewards.epochs_completed.toString()"
        />
        <StatRow
          label="Avg Per Epoch"
          :value="avgPerEpoch"
          color="yellow"
        />
      </div>

      <!-- Rewards Timeline (placeholder) -->
      <div class="mt-6 pt-4 border-t border-vault-accent">
        <div class="text-sm text-gray-400 mb-2">Reward History</div>
        <div class="h-16 bg-vault-accent rounded flex items-end justify-around p-2">
          <div v-for="i in 10" :key="i"
               class="w-4 bg-vault-green rounded-t"
               :style="{ height: (20 + Math.random() * 80) + '%' }">
          </div>
        </div>
      </div>
    </div>

    <!-- Gateway Stats (if available) -->
    <div v-if="snapshot.economics.gateway" class="card">
      <h2 class="text-lg font-semibold text-vault-cyan mb-4">Gateway Revenue</h2>

      <div class="text-center mb-6">
        <div class="text-4xl font-bold text-vault-green">
          {{ formatTokens(snapshot.economics.gateway.total_revenue) }}
        </div>
        <div class="text-gray-400 mt-1">Total Revenue</div>
      </div>

      <div class="grid grid-cols-2 gap-4 mb-6">
        <div class="text-center p-4 bg-vault-accent rounded-lg">
          <div class="text-2xl font-bold text-vault-cyan">{{ snapshot.economics.gateway.active_clients }}</div>
          <div class="text-sm text-gray-400">Active Clients</div>
        </div>
        <div class="text-center p-4 bg-vault-accent rounded-lg">
          <div class="text-2xl font-bold text-white">{{ formatNumber(snapshot.economics.gateway.requests_served) }}</div>
          <div class="text-sm text-gray-400">Requests Served</div>
        </div>
      </div>

      <div class="space-y-4">
        <div>
          <div class="flex justify-between mb-2">
            <span class="text-gray-400">Current Load</span>
            <span :class="loadColor">{{ (snapshot.economics.gateway.current_load * 100).toFixed(1) }}%</span>
          </div>
          <div class="gauge-bar h-3">
            <div
              class="gauge-fill"
              :class="loadGaugeClass"
              :style="{ width: (snapshot.economics.gateway.current_load * 100) + '%' }"
            ></div>
          </div>
        </div>
        <StatRow
          label="Avg Revenue/Request"
          :value="avgRevenuePerRequest"
        />
      </div>
    </div>

    <!-- Performance Summary -->
    <div class="card" :class="{ 'lg:col-span-2': !snapshot.economics.gateway }">
      <h2 class="text-lg font-semibold text-vault-cyan mb-4">Performance Summary</h2>

      <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
        <div class="text-center p-4 bg-vault-accent rounded-lg">
          <div class="text-2xl font-bold" :class="performanceColorClass">
            {{ snapshot.economics.staking.performance_score.toFixed(2) }}x
          </div>
          <div class="text-sm text-gray-400">Performance</div>
        </div>
        <div class="text-center p-4 bg-vault-accent rounded-lg">
          <div class="text-2xl font-bold text-vault-cyan">
            {{ snapshot.consensus.reputation_score.toFixed(2) }}
          </div>
          <div class="text-sm text-gray-400">Reputation</div>
        </div>
        <div class="text-center p-4 bg-vault-accent rounded-lg">
          <div class="text-2xl font-bold text-vault-green">
            {{ snapshot.storage.compression_ratio.toFixed(1) }}:1
          </div>
          <div class="text-sm text-gray-400">Compression</div>
        </div>
        <div class="text-center p-4 bg-vault-accent rounded-lg">
          <div class="text-2xl font-bold text-vault-yellow">
            {{ (snapshot.storage.cache_hit_rate * 100).toFixed(1) }}%
          </div>
          <div class="text-sm text-gray-400">Cache Hit Rate</div>
        </div>
      </div>

      <div class="mt-6 p-4 bg-vault-accent rounded-lg">
        <div class="text-sm text-gray-400 mb-2">Estimated Daily Earnings</div>
        <div class="text-2xl font-bold text-vault-green">
          {{ estimatedDailyEarnings }}
        </div>
        <div class="text-xs text-gray-500 mt-1">
          Based on current stake and performance score
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

const stakeShare = computed(() => {
  if (!props.snapshot.economics.staking.total_staked) return '0%'
  const share = (props.snapshot.economics.staking.own_stake / props.snapshot.economics.staking.total_staked) * 100
  return share.toFixed(4) + '%'
})

const performanceColor = computed(() => {
  const score = props.snapshot.economics.staking.performance_score
  if (score >= 1) return 'green'
  if (score >= 0.5) return 'yellow'
  return 'red'
})

const performanceColorClass = computed(() => {
  const score = props.snapshot.economics.staking.performance_score
  if (score >= 1) return 'text-vault-green'
  if (score >= 0.5) return 'text-vault-yellow'
  return 'text-vault-red'
})

const effectiveApy = computed(() => {
  const base = props.snapshot.economics.staking.base_apy
  const perf = props.snapshot.economics.staking.performance_score
  return ((base * perf) * 100).toFixed(2) + '%'
})

const avgPerEpoch = computed(() => {
  if (!props.snapshot.economics.rewards.epochs_completed) return 'N/A'
  return formatTokens(props.snapshot.economics.rewards.total_earned / props.snapshot.economics.rewards.epochs_completed)
})

const loadColor = computed(() => {
  const load = props.snapshot.economics.gateway?.current_load || 0
  if (load < 0.7) return 'text-vault-green'
  if (load < 0.9) return 'text-vault-yellow'
  return 'text-vault-red'
})

const loadGaugeClass = computed(() => {
  const load = props.snapshot.economics.gateway?.current_load || 0
  if (load < 0.7) return 'bg-vault-green'
  if (load < 0.9) return 'bg-vault-yellow'
  return 'bg-vault-red'
})

const avgRevenuePerRequest = computed(() => {
  const gateway = props.snapshot.economics.gateway
  if (!gateway || !gateway.requests_served) return 'N/A'
  return (gateway.total_revenue / gateway.requests_served).toFixed(2)
})

const estimatedDailyEarnings = computed(() => {
  const stake = props.snapshot.economics.staking.own_stake
  const apy = props.snapshot.economics.staking.base_apy
  const perf = props.snapshot.economics.staking.performance_score
  const daily = (stake * apy * perf) / 365
  return formatTokens(Math.round(daily))
})

function formatTokens(amount) {
  if (amount >= 1e9) return (amount / 1e9).toFixed(2) + 'B'
  if (amount >= 1e6) return (amount / 1e6).toFixed(2) + 'M'
  if (amount >= 1e3) return (amount / 1e3).toFixed(2) + 'K'
  return amount.toString()
}

function formatNumber(num) {
  if (num >= 1e9) return (num / 1e9).toFixed(2) + 'B'
  if (num >= 1e6) return (num / 1e6).toFixed(2) + 'M'
  if (num >= 1e3) return (num / 1e3).toFixed(2) + 'K'
  return num.toString()
}
</script>
