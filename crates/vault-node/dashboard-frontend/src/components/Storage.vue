<template>
  <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
    <!-- Storage Stats -->
    <div class="card">
      <h2 class="text-lg font-semibold text-vault-cyan mb-4">Storage Statistics</h2>
      <div class="space-y-4">
        <StatRow label="Total Capacity" :value="formatBytes(snapshot.storage.total_capacity)" />
        <StatRow label="Used Capacity" :value="formatBytes(snapshot.storage.used_capacity)" color="cyan" />
        <StatRow label="Available" :value="formatBytes(snapshot.storage.available_capacity)" color="green" />
        <StatRow label="Usage %" :value="usagePercent.toFixed(1) + '%'" :color="usageColor" />
        <StatRow label="Blocks Stored" :value="snapshot.storage.blocks_stored.toString()" color="yellow" />
      </div>

      <!-- Usage Gauge -->
      <div class="mt-6">
        <div class="text-sm text-gray-400 mb-2">Storage Usage</div>
        <div class="gauge-bar h-6">
          <div
            class="gauge-fill"
            :class="gaugeClass"
            :style="{ width: usagePercent + '%' }"
          ></div>
        </div>
      </div>
    </div>

    <!-- Compression Stats -->
    <div class="card">
      <h2 class="text-lg font-semibold text-vault-cyan mb-4">Compression Statistics</h2>
      <div class="space-y-4">
        <div class="text-center mb-6">
          <div class="text-5xl font-bold text-vault-green">{{ snapshot.storage.compression_ratio.toFixed(2) }}:1</div>
          <div class="text-gray-400 mt-1">Average Compression Ratio</div>
        </div>

        <StatRow label="Original Size" :value="formatBytes(snapshot.storage.total_original_bytes)" />
        <StatRow label="Compressed Size" :value="formatBytes(snapshot.storage.total_compressed_bytes)" color="cyan" />
        <StatRow label="Space Saved" :value="formatBytes(spaceSaved)" color="green" />
        <StatRow label="Savings %" :value="savingsPercent.toFixed(1) + '%'" color="green" />
      </div>
    </div>

    <!-- Cache Stats -->
    <div class="card">
      <h2 class="text-lg font-semibold text-vault-cyan mb-4">Cache Performance</h2>
      <div class="grid grid-cols-3 gap-4 text-center mb-6">
        <div>
          <div class="text-3xl font-bold text-vault-green">{{ snapshot.storage.cache_hits }}</div>
          <div class="text-sm text-gray-400">Hits</div>
        </div>
        <div>
          <div class="text-3xl font-bold text-vault-red">{{ snapshot.storage.cache_misses }}</div>
          <div class="text-sm text-gray-400">Misses</div>
        </div>
        <div>
          <div class="text-3xl font-bold text-vault-cyan">{{ cacheHitRate.toFixed(1) }}%</div>
          <div class="text-sm text-gray-400">Hit Rate</div>
        </div>
      </div>

      <!-- Cache Hit Rate Bar -->
      <div>
        <div class="text-sm text-gray-400 mb-2">Cache Efficiency</div>
        <div class="gauge-bar h-4">
          <div
            class="gauge-fill bg-vault-cyan"
            :style="{ width: cacheHitRate + '%' }"
          ></div>
        </div>
      </div>
    </div>

    <!-- Block Info -->
    <div class="card">
      <h2 class="text-lg font-semibold text-vault-cyan mb-4">Block Statistics</h2>
      <div class="space-y-4">
        <StatRow label="Total Blocks" :value="snapshot.storage.blocks_stored.toString()" />
        <StatRow
          label="Avg Original Size"
          :value="avgOriginalSize"
        />
        <StatRow
          label="Avg Compressed Size"
          :value="avgCompressedSize"
          color="cyan"
        />
        <StatRow
          label="Avg Savings/Block"
          :value="avgSavingsPerBlock"
          color="green"
        />
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

const usagePercent = computed(() => {
  if (!props.snapshot?.storage?.total_capacity) return 0
  return (props.snapshot.storage.used_capacity / props.snapshot.storage.total_capacity) * 100
})

const usageColor = computed(() => {
  if (usagePercent.value < 70) return 'green'
  if (usagePercent.value < 90) return 'yellow'
  return 'red'
})

const gaugeClass = computed(() => {
  if (usagePercent.value < 70) return 'bg-vault-cyan'
  if (usagePercent.value < 90) return 'bg-vault-yellow'
  return 'bg-vault-red'
})

const spaceSaved = computed(() => {
  return props.snapshot.storage.total_original_bytes - props.snapshot.storage.total_compressed_bytes
})

const savingsPercent = computed(() => {
  if (!props.snapshot?.storage?.total_original_bytes) return 0
  return (spaceSaved.value / props.snapshot.storage.total_original_bytes) * 100
})

const cacheHitRate = computed(() => {
  return props.snapshot?.storage?.cache_hit_rate * 100 || 0
})

const avgOriginalSize = computed(() => {
  if (!props.snapshot?.storage?.blocks_stored) return 'N/A'
  return formatBytes(props.snapshot.storage.total_original_bytes / props.snapshot.storage.blocks_stored)
})

const avgCompressedSize = computed(() => {
  if (!props.snapshot?.storage?.blocks_stored) return 'N/A'
  return formatBytes(props.snapshot.storage.total_compressed_bytes / props.snapshot.storage.blocks_stored)
})

const avgSavingsPerBlock = computed(() => {
  if (!props.snapshot?.storage?.blocks_stored) return 'N/A'
  return formatBytes(spaceSaved.value / props.snapshot.storage.blocks_stored)
})

function formatBytes(bytes) {
  if (bytes >= 1e12) return (bytes / 1e12).toFixed(2) + ' TB'
  if (bytes >= 1e9) return (bytes / 1e9).toFixed(2) + ' GB'
  if (bytes >= 1e6) return (bytes / 1e6).toFixed(2) + ' MB'
  if (bytes >= 1e3) return (bytes / 1e3).toFixed(2) + ' KB'
  return Math.round(bytes) + ' B'
}
</script>
