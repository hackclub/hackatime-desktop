<template>
  <div class="bg-bg-card border border-border-primary rounded-2xl p-6 shadow-card">
    <div class="flex items-start space-x-4">
      <div class="text-3xl">{{ icon }}</div>
      <div class="flex-1">
        <h3 class="text-lg font-semibold text-text-primary mb-2">{{ title }}</h3>
        <p class="text-text-secondary mb-3">{{ description }}</p>
        <div class="flex items-center justify-between">
          <div class="text-2xl font-bold" :style="{ color: color }">{{ value }}</div>
          <div class="text-sm text-text-secondary">{{ trend }}</div>
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
