<template>
  <div class="rounded-2xl shadow-card p-6" style="background-color: #191415;">
    <!-- Presence Title -->
    <h3 class="text-text-primary font-semibold text-lg mb-4">Current Session</h3>
    
    <!-- Loading State -->
    <div v-if="isLoading" class="text-center py-8">
      <div class="w-16 h-16 bg-bg-secondary rounded-full flex items-center justify-center mx-auto mb-4">
        <div class="w-6 h-6 border-2 border-text-secondary border-t-transparent rounded-full animate-spin"></div>
      </div>
      <div class="text-text-secondary text-lg font-medium mb-2">Loading session data...</div>
    </div>
    
    <!-- Active Session Display -->
    <div v-else-if="sessionState.is_active" class="space-y-4">
      <!-- Project and Editor Info -->
      <div class="flex items-center justify-between">
        <div class="flex items-center space-x-3">
          <div class="w-3 h-3 bg-green-500 rounded-full animate-pulse"></div>
          <div>
            <div class="text-text-primary font-medium text-lg">
              {{ sessionState.project || 'Unknown Project' }}
            </div>
            <div class="text-text-secondary text-sm">
              {{ sessionState.editor || 'Unknown Editor' }}
            </div>
          </div>
        </div>
        <div class="text-right">
          <div class="text-text-secondary text-sm">Language</div>
          <div class="text-text-primary font-medium">
            {{ sessionState.language || 'Unknown' }}
          </div>
        </div>
      </div>
      
      <!-- File being worked on -->
      <div v-if="sessionState.entity" class="bg-bg-secondary rounded-lg p-3">
        <div class="text-text-secondary text-xs mb-1">Currently editing</div>
        <div class="text-text-primary font-mono text-sm truncate">
          {{ sessionState.entity }}
        </div>
      </div>
      
      <!-- Session duration -->
      <div class="flex items-center justify-between text-sm">
        <div class="text-text-secondary">Session started</div>
        <div class="text-text-primary font-medium">
          {{ formatTime(sessionState.start_time) }}
        </div>
      </div>
      
      <!-- Heartbeat count -->
      <div class="flex items-center justify-between text-sm">
        <div class="text-text-secondary">Heartbeats</div>
        <div class="text-text-primary font-medium">
          {{ sessionState.heartbeat_count }}
        </div>
      </div>
    </div>
    
    <!-- No active session -->
    <div v-else class="text-center py-8">
      <div class="w-16 h-16 bg-bg-secondary rounded-full flex items-center justify-center mx-auto mb-4">
        <svg class="w-8 h-8 text-text-secondary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z"></path>
        </svg>
      </div>
      <div class="text-text-secondary text-lg font-medium mb-2">No active coding session</div>
      <div class="text-text-secondary text-sm">Start coding in your editor to see your current session here</div>
      <div class="text-text-secondary text-xs mt-2 opacity-75">Make sure your editor has the WakaTime plugin installed and configured</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface AuthState {
  is_authenticated: boolean;
  access_token: string | null;
  user_info: Record<string, any> | null;
}

interface SessionState {
  is_active: boolean;
  start_time: number | null;
  last_heartbeat_id: number | null;
  heartbeat_count: number;
  project: string | null;
  editor: string | null;
  language: string | null;
  entity: string | null;
}

const props = defineProps<{
  authState: AuthState;
  presenceData: any;
  apiConfig: any;
}>();

const sessionState = ref<SessionState>({
  is_active: false,
  start_time: null,
  last_heartbeat_id: null,
  heartbeat_count: 0,
  project: null,
  editor: null,
  language: null,
  entity: null,
});

const isLoading = ref(true);
let sessionRefreshInterval: number | null = null;


function formatTime(timestamp: number | null): string {
  if (!timestamp) return 'Unknown';
  
  const now = Math.floor(Date.now() / 1000);
  const diff = now - timestamp;
  
  if (diff < 60) {
    return 'Just now';
  } else if (diff < 3600) {
    const minutes = Math.floor(diff / 60);
    return `${minutes}m ago`;
  } else if (diff < 86400) {
    const hours = Math.floor(diff / 3600);
    return `${hours}h ago`;
  } else {
    const days = Math.floor(diff / 86400);
    return `${days}d ago`;
  }
}

async function loadSessionState() {
  if (!props.authState.is_authenticated) {
    isLoading.value = false;
    return;
  }
  
  try {
    
    const session = await invoke("get_current_session");
    console.log("Session state loaded:", session);
    sessionState.value = session as SessionState;
    isLoading.value = false;
  } catch (error) {
    console.error("Failed to load session state:", error);
    isLoading.value = false;
  }
}

function startSessionRefresh() {
  if (sessionRefreshInterval) {
    clearInterval(sessionRefreshInterval);
  }
  
  
  sessionRefreshInterval = setInterval(loadSessionState, 10000);
}

function stopSessionRefresh() {
  if (sessionRefreshInterval) {
    clearInterval(sessionRefreshInterval);
    sessionRefreshInterval = null;
  }
}


watch(() => props.presenceData, () => {
  if (props.authState.is_authenticated) {
    loadSessionState();
  }
}, { deep: true });

onMounted(() => {
  loadSessionState();
  startSessionRefresh();
});

onUnmounted(() => {
  stopSessionRefresh();
});
</script>

<!-- All styles now handled by Tailwind CSS -->