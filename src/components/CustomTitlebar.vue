<template>
  <div class="custom-titlebar" :class="{ 'macOS': isMacOS }">
    <!-- macOS Traffic Lights (left side on macOS) -->
    <div v-if="isMacOS" class="macos-traffic-lights">
      <button 
        class="traffic-light close-light" 
        @click="closeWindow"
        title="Close"
        aria-label="Close"
      >
        <svg width="8" height="8" viewBox="0 0 8 8" class="traffic-light-icon">
          <path d="M1 1L7 7M7 1L1 7" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        </svg>
      </button>
      <button 
        class="traffic-light minimize-light" 
        @click="minimizeWindow"
        title="Minimize"
        aria-label="Minimize"
      >
        <svg width="8" height="2" viewBox="0 0 8 2" class="traffic-light-icon">
          <rect width="8" height="1.5" fill="currentColor"/>
        </svg>
      </button>
      <button 
        class="traffic-light maximize-light" 
        @click="toggleMaximize"
        title="Maximize"
        aria-label="Maximize"
      >
        <svg width="8" height="8" viewBox="0 0 8 8" class="traffic-light-icon">
          <path d="M1 3L4 6L7 3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" fill="none"/>
          <path d="M1 5L4 2L7 5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" fill="none"/>
        </svg>
      </button>
    </div>

    <!-- Window controls (left side on Windows/Linux, hidden on macOS) -->
    <div v-if="!isMacOS" class="titlebar-controls">
      <!-- Minimize button -->
      <button 
        class="titlebar-button minimize-button" 
        @click="minimizeWindow"
        title="Minimize"
      >
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/svg">
          <rect x="2" y="5.5" width="8" height="1" fill="currentColor"/>
        </svg>
      </button>

      <!-- Maximize/Restore button -->
      <button 
        class="titlebar-button maximize-button" 
        @click="toggleMaximize"
        title="Maximize"
      >
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/svg">
          <rect x="1" y="1" width="10" height="10" stroke="currentColor" stroke-width="1" fill="none"/>
        </svg>
      </button>

      <!-- Close button -->
      <button 
        class="titlebar-button close-button" 
        @click="closeWindow"
        title="Close"
      >
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path d="M3 3L9 9M9 3L3 9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        </svg>
      </button>
    </div>

    <!-- Drag region - Center area with app title -->
    <div class="titlebar-drag-region" data-tauri-drag-region @dblclick="handleDoubleClick">
      <div class="app-title">
        <svg class="app-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path d="M12 2L2 7L12 12L22 7L12 2Z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
          <path d="M2 17L12 22L22 17" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
          <path d="M2 12L12 17L22 12" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
        <span class="app-name">Hackatime Desktop</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';

const isMacOS = ref(false);

onMounted(() => {
  isMacOS.value = navigator.userAgent.includes('Mac');
});

const handleDoubleClick = async () => {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    await getCurrentWindow().toggleMaximize();
  } catch (error) {
    console.error('Failed to toggle maximize:', error);
  }
};

const minimizeWindow = async () => {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    await getCurrentWindow().minimize();
  } catch (error) {
    console.error('Failed to minimize window:', error);
  }
};

const toggleMaximize = async () => {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    await getCurrentWindow().toggleMaximize();
  } catch (error) {
    console.error('Failed to toggle maximize:', error);
  }
};

const closeWindow = async () => {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    await getCurrentWindow().close();
  } catch (error) {
    console.error('Failed to close window:', error);
  }
};
</script>

<style scoped>
.custom-titlebar {
  height: 56px;
  background-color: #59405C;
  border-bottom: 4px solid #47334A;
  display: flex;
  align-items: center;
  user-select: none;
  position: relative;
  z-index: 1000;
  padding: 0 16px;
  border-radius: 12px 12px 0 0;
}

/* macOS Traffic Lights */
.macos-traffic-lights {
  display: flex;
  gap: 8px;
  padding: 0 12px;
  align-items: center;
}

.traffic-light {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  border: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
  padding: 0;
}

.traffic-light-icon {
  opacity: 0;
  transition: opacity 0.2s ease;
}

.traffic-light:hover .traffic-light-icon {
  opacity: 1;
}

.close-light {
  background-color: #ff5f57;
}

.close-light:hover {
  background-color: #ff4841;
}

.close-light .traffic-light-icon {
  color: #6e0a00;
}

.minimize-light {
  background-color: #ffbd2e;
}

.minimize-light:hover {
  background-color: #ffaa00;
}

.minimize-light .traffic-light-icon {
  color: #8b5d00;
}

.maximize-light {
  background-color: #28c940;
}

.maximize-light:hover {
  background-color: #1fb835;
}

.maximize-light .traffic-light-icon {
  color: #0d5917;
}

.titlebar-drag-region {
  flex: 1;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* On macOS, align to left */
.macOS .titlebar-drag-region {
  justify-content: flex-start;
}

.app-title {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #ffffff;
  font-size: 14px;
  font-weight: 500;
}

.app-icon {
  color: #E99682;
  flex-shrink: 0;
}

.app-name {
  font-family: 'Outfit', sans-serif;
}

.titlebar-controls {
  display: flex;
  align-items: center;
  gap: 4px;
}

.titlebar-button {
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: #ffffff;
  transition: background-color 0.2s ease;
  -webkit-app-region: no-drag;
}

.titlebar-button:hover {
  background-color: rgba(255, 255, 255, 0.1);
}

.minimize-button:hover {
  background-color: rgba(255, 255, 255, 0.2);
}

.maximize-button:hover {
  background-color: rgba(255, 255, 255, 0.2);
}

.close-button:hover {
  background-color: #e81123;
  color: #ffffff;
}

.titlebar-button:active {
  background-color: rgba(255, 255, 255, 0.3);
}

.close-button:active {
  background-color: #c50e1f;
}

</style>
