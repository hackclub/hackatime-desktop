<script setup lang="ts">
import { ref, computed } from 'vue';

const props = defineProps<{
  version: string;
}>();

const emit = defineEmits<{
  installNow: [];
  moreInfo: [];
  dismiss: [];
}>();

const isVisible = ref(true);

const handleInstallNow = () => {
  emit('installNow');
};

const handleMoreInfo = () => {
  emit('moreInfo');
};

const handleDismiss = () => {
  isVisible.value = false;
  emit('dismiss');
};
</script>

<template>
  <transition
    enter-active-class="transition-all duration-300 ease-out"
    enter-from-class="translate-x-full opacity-0"
    enter-to-class="translate-x-0 opacity-100"
    leave-active-class="transition-all duration-200 ease-in"
    leave-from-class="translate-x-0 opacity-100"
    leave-to-class="translate-x-full opacity-0"
  >
    <div 
      v-if="isVisible"
      class="card-3d-update pointer-events-auto"
      style="position: fixed; z-index: 9999; bottom: 20px; right: 20px; width: 320px;"
    >
      <div class="rounded-lg border-2 border-black p-4 card-3d-update-front" style="background-color: #3D2C3E;">
        <div class="flex items-start justify-between mb-3">
          <div class="flex items-center gap-2">
            <svg class="w-5 h-5 text-[#EB9182]" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm1-11a1 1 0 10-2 0v3.586L7.707 9.293a1 1 0 00-1.414 1.414l3 3a1 1 0 001.414 0l3-3a1 1 0 00-1.414-1.414L11 10.586V7z" clip-rule="evenodd"></path>
            </svg>
            <h3 class="text-white text-sm font-bold m-0" style="font-family: 'Outfit', sans-serif;">
              Update Available
            </h3>
          </div>
          <button 
            @click="handleDismiss"
            class="text-white/60 hover:text-white transition-colors"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
            </svg>
          </button>
        </div>

        <p class="text-white/80 text-xs mb-4 leading-relaxed" style="font-family: 'Outfit', sans-serif;">
          Version <span class="font-semibold text-[#EB9182]">{{ version }}</span> is ready to install
        </p>

        <div class="flex gap-2">
          <button
            @click="handleMoreInfo"
            class="flex-1 px-3 py-2 rounded-lg text-xs font-medium transition-all duration-200"
            style="background-color: #543c55; color: white; font-family: 'Outfit', sans-serif; border: 1px solid rgba(0,0,0,0.3);"
          >
            More Info
          </button>
          <button
            @click="handleInstallNow"
            class="flex-1 px-3 py-2 rounded-lg text-xs font-bold transition-all duration-200"
            style="background: linear-gradient(135deg, #E99682 0%, #EB9182 33%, #E88592 66%, #E883AE 100%); color: white; font-family: 'Outfit', sans-serif; border: 1px solid rgba(0,0,0,0.3);"
          >
            Install Now
          </button>
        </div>
      </div>
    </div>
  </transition>
</template>

<style scoped>
.card-3d-update {
  position: relative;
  border-radius: 8px;
  padding: 0;
}

.card-3d-update::before {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: 8px;
  background-color: #2A1F2B;
  z-index: 0;
}

.card-3d-update-front {
  position: relative;
  transform: translateY(-4px);
  z-index: 1;
}

button:hover {
  transform: translateY(-1px);
}

button:active {
  transform: translateY(0);
}
</style>

