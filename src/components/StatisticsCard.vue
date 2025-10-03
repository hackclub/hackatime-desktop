<template>
  <div class="bg-bg-card border border-border-primary rounded-2xl p-6 shadow-card">
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-lg font-semibold text-text-primary">{{ title }}</h3>
      <div class="text-2xl">{{ icon }}</div>
    </div>
    
    <div class="space-y-4">
      <div class="flex items-center justify-between">
        <div>
          <div class="text-3xl font-bold text-text-primary">{{ value }}</div>
          <div class="text-sm text-text-secondary">{{ period }}</div>
        </div>
        <div class="text-right">
          <div 
            class="text-sm font-medium px-3 py-1 rounded-full"
            :class="changeClass"
          >
            {{ change }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

interface Props {
  title: string;
  value: string;
  change: string;
  changeType: string;
  period: string;
  icon: string;
  color: string;
}

const props = defineProps<Props>();

const changeClass = computed(() => {
  switch (props.changeType) {
    case 'increase':
      return 'bg-green-100 text-green-800';
    case 'decrease':
      return 'bg-red-100 text-red-800';
    case 'neutral':
    default:
      return 'bg-gray-100 text-gray-800';
  }
});
</script>
