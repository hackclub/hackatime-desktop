<template>
  <div class="rounded-xl p-4 flex items-center space-x-3 relative overflow-hidden" :style="cardStyle">
    <div class="text-2xl">{{ icon }}</div>
    <div class="flex-1">
      <div class="text-sm text-white font-medium">{{ title }}</div>
      <div class="text-xs text-white/70">{{ period }}</div>
    </div>
    <div class="text-right">
      <div 
        class="text-2xl font-bold px-3 py-2 rounded-lg"
        :class="changeClass"
      >
        {{ change }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

interface Props {
  title: string;
  change: string;
  changeType: string;
  period: string;
  icon: string;
}

const props = defineProps<Props>();

const cardStyle = computed(() => {
  switch (props.changeType) {
    case 'increase':
      return {
        background: 'linear-gradient(135deg, #FB4C21 0%, #FF6B35 100%)',
        border: '1px solid #FB4C21'
      };
    case 'decrease':
      return {
        background: 'linear-gradient(135deg, #4568DC 0%, #5A7BE8 100%)',
        border: '1px solid #4568DC'
      };
    case 'neutral':
    default:
      return {
        background: 'linear-gradient(135deg, #6B7280 0%, #9CA3AF 100%)',
        border: '1px solid #6B7280'
      };
  }
});

const changeClass = computed(() => {
  switch (props.changeType) {
    case 'increase':
      return 'bg-white/20 text-white';
    case 'decrease':
      return 'bg-white/20 text-white';
    case 'neutral':
    default:
      return 'bg-white/20 text-white';
  }
});
</script>
