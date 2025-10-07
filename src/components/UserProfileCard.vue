<template>
  <div class="card-3d">
    <div class="rounded-[8px] border-2 border-black p-4 card-3d-front" style="background: #f9e9b5;">
      <!-- Header with User Info -->
      <div class="flex items-center gap-3 mb-3">
        <div class="relative">
          <img 
            :src="gravatarUrl" 
            :alt="userName" 
            class="w-10 h-10 rounded-md border-2 border-[#594d37] bg-white"
          />
        </div>
        <div class="flex-1 min-w-0">
          <div class="font-bold text-base leading-tight" style="color: #594d37; font-family: 'Outfit', sans-serif;">
            {{ userName }}
          </div>
        </div>
      </div>

      <!-- Current Session Section -->
      <div class="mb-3">
        <!-- Loading State -->
        <div v-if="isLoading" class="rounded-md p-3 text-center" style="background-color: rgba(89, 77, 55, 0.1);">
          <div class="w-4 h-4 border-2 border-[#594d37] border-t-transparent rounded-full animate-spin mx-auto"></div>
          <div class="text-xs font-medium mt-1" style="color: #594d37; font-family: 'Outfit', sans-serif;">Loading...</div>
        </div>
        
        <!-- Active Session Display -->
        <div v-else-if="sessionState.is_active" class="rounded-md p-3 space-y-2" style="background-color: rgba(89, 77, 55, 0.1);">
          <div v-if="sessionState.project" class="flex items-center gap-2 text-xs font-medium" style="color: #594d37; font-family: 'Outfit', sans-serif;">
            <span class="text-sm">📁</span>
            <span class="truncate">{{ sessionState.project }}</span>
          </div>
          <div v-if="sessionState.language" class="flex items-center gap-2 text-xs font-medium" style="color: #594d37; font-family: 'Outfit', sans-serif;">
            <span class="text-sm">💻</span>
            <span class="truncate">{{ sessionState.language }}</span>
          </div>
          <div v-if="sessionState.entity" class="flex items-center gap-2 text-xs font-medium opacity-80" style="color: #594d37; font-family: 'Outfit', sans-serif;">
            <span class="text-sm">📄</span>
            <span class="truncate">{{ sessionState.entity.split('/').pop() || sessionState.entity }}</span>
          </div>
        </div>
        
        <!-- No active session -->
        <div v-else class="rounded-md p-3 text-center" style="background-color: rgba(89, 77, 55, 0.1);">
          <div class="text-lg mb-1">💤</div>
          <div class="text-xs font-medium" style="color: #594d37; font-family: 'Outfit', sans-serif;">No active session</div>
        </div>
      </div>

      <!-- Settings Button - Bottom right corner -->
      <div class="flex justify-end">
        <button 
          @click="$emit('openSettings')"
          class="w-7 h-7 flex items-center justify-center rounded-md transition-all duration-200 cursor-pointer hover:scale-105"
          style="background: rgba(89, 77, 55, 0.1);"
          title="Settings"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="#594d37" viewBox="0 0 24 24" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"></path>
            <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"></path>
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import CryptoJS from "crypto-js";

interface AuthState {
  is_authenticated: boolean;
  access_token: string | null;
  user_info: Record<string, any> | null;
}

interface UserData {
  emails: string[];
  slack_id: string;
  trust_factor: {
    trust_level: string;
    trust_value: number;
  };
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
  userData: UserData | null;
  presenceData: any;
  apiConfig: any;
}>();

defineEmits<{
  openSettings: [];
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


const userEmail = computed(() => {
  if (!props.userData?.emails || props.userData.emails.length === 0) {
    return 'user@example.com';
  }
  return props.userData.emails[0];
});

const userName = computed(() => {
  
  const email = userEmail.value;
  return email.split('@')[0];
});




const gravatarUrl = computed(() => {
  const email = userEmail.value.trim().toLowerCase();
  const hash = CryptoJS.MD5(email).toString();
  return `https://www.gravatar.com/avatar/${hash}?d=identicon&s=128`;
});



async function loadSessionState() {
  if (!props.authState.is_authenticated) {
    isLoading.value = false;
    return;
  }
  
  try {
    const session = await invoke("get_current_session");
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
  background: #d4c48a; 
  z-index: 0; 
}

.card-3d-front { 
  position: relative; 
  transform: translateY(-6px); 
  z-index: 1; 
}
</style>
