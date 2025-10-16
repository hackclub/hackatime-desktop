<template>
  <div class="card-3d">
    <div class="rounded-[8px] border border-black p-6 card-3d-front h-full flex flex-col" style="background-color: #3D2C3E;">
    <div class="flex items-start space-x-4">
      <div class="text-3xl flex-shrink-0">{{ icon }}</div>
      <div class="flex-1 flex flex-col min-w-0">
        <h3 class="text-lg font-semibold text-text-primary mb-2">{{ title }}</h3>
        <p class="text-text-secondary mb-4 text-sm line-clamp-2">{{ description }}</p>
        <div class="mt-auto">
          <div class="text-2xl font-bold mb-1" :style="{ color: color }">{{ value }}</div>
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
