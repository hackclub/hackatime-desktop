<template>
  <div class="flex flex-col h-full min-h-0">
    <!-- Header styling aligned to Home.vue -->
    <div class="mb-6">
      <h1 class="text-[40px] sm:text-[32px] lg:text-[40px] font-bold italic text-white m-0 mb-2" style="font-family: 'Outfit', sans-serif;">
        statistics
      </h1>
      <p class="text-[20px] sm:text-[16px] lg:text-[20px] text-white m-0" style="font-family: 'Outfit', sans-serif;">
        your insights at a glance.
      </p>
    </div>

    <!-- Statistics Dashboard in 3D card style -->
    <div class="card-3d card-3d-stats">
      <div class="rounded-[8px] border border-black p-6 card-3d-front" style="background-color: #3D2C3E;">
        <StatisticsDashboard :apiConfig="apiConfig" />
      </div>
    </div>
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

<style scoped>
.card-3d {
  position: relative;
  border-radius: 8px;
  padding: 0;
}

.card-3d::before {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: 8px;
  background: #2A1F2B;
  z-index: 0;
}

.card-3d-stats::before {
  background: #2A1F2B !important;
}

.card-3d-front {
  position: relative;
  transform: translateY(-6px);
  z-index: 1;
  box-shadow: 0 6px 0 #2A1F2B;
}
</style>
