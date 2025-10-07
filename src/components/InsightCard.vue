<template>
  <div class="card-3d">
    <div class="rounded-[8px] border-2 border-black p-6 card-3d-front h-full flex flex-col" style="background-color: #3D2C3E;">
    <div class="flex items-start space-x-4 flex-1">
      <div class="text-3xl">{{ icon }}</div>
      <div class="flex-1 flex flex-col">
        <h3 class="text-lg font-semibold text-text-primary mb-2">{{ title }}</h3>
        <p class="text-text-secondary mb-3 flex-1">{{ description }}</p>
        <div class="flex items-center justify-between">
          <div class="text-2xl font-bold" :style="{ color: color }">{{ value }}</div>
          <div class="text-sm text-text-secondary">{{ trend }}</div>
        </div>
      </div>
    </div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  title: string;
  description: string;
  value: string;
  trend: string;
  icon: string;
  color: string;
}

defineProps<Props>();

</script>

<style scoped>
.card-3d { position: relative; border-radius: 8px; padding: 0; }
.card-3d::before { content: ''; position: absolute; inset: 0; border-radius: 8px; background: #2A1F2B; z-index: 0; }
.card-3d-front { position: relative; transform: translateY(-6px); z-index: 1; }
</style>
