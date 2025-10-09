<template>
  <div class="card-3d">
    <div class="rounded-[8px] border border-black p-6 card-3d-front h-full flex flex-col" style="background-color: #3D2C3E;">
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-lg font-semibold text-text-primary">{{ title }}</h3>
      <div class="text-2xl">{{ icon }}</div>
    </div>
    
    <div class="space-y-4 flex-1">
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
      return 'bg-[rgba(34,197,94,0.15)] text-[#22c55e]';
    case 'decrease':
      return 'bg-[rgba(236,59,72,0.15)] text-[#ec3b48]';
    case 'neutral':
    default:
      return 'bg-[rgba(255,255,255,0.08)] text-[#f5e6e8]';
  }
});
</script>

<style scoped>
.card-3d { position: relative; border-radius: 8px; padding: 0; }
.card-3d::before { content: ''; position: absolute; inset: 0; border-radius: 8px; background: #2A1F2B; z-index: 0; }
.card-3d-front { position: relative; transform: translateY(-6px); z-index: 1; }
</style>
