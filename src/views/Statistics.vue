<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between mb-6">
      <h1 class="text-2xl font-bold text-text-primary">Statistics</h1>
      <div class="text-sm text-text-secondary">Your coding insights and trends</div>
    </div>

    <!-- Statistics Dashboard -->
    <StatisticsDashboard :apiConfig="apiConfig" />
  </div>
</template>

<script setup lang="ts">
import StatisticsDashboard from '../components/StatisticsDashboard.vue';

interface ApiConfig {
  base_url: string;
}

defineProps<{
  apiConfig: ApiConfig;
}>();
</script>
