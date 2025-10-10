<template>
  <div class="flex items-center justify-center h-full w-full" style="background-color: #322433;">
    <div class="max-w-md w-full px-8">
      <div v-if="!authInProgress" class="flex justify-center mb-8">
        <img src="/src/assets/bird-illustration.svg" alt="Hackatime" class="h-24 w-auto" />
      </div>
      <div class="card-3d">
        <div class="rounded-[12px] border-2 border-black card-3d-front p-8" style="background-color: #3D2C3E;">
          <div class="text-center">
            <h1 v-if="!authInProgress" class="text-[32px] font-bold text-white mb-8" style="font-family: 'Outfit', sans-serif;">
              Welcome to Hackatime
            </h1>
            <div v-if="!authInProgress">
              <button 
                @click="handleLogin" 
                :disabled="isLoading"
                class="pushable w-full mt-8"
                :class="isLoading ? '' : 'pushable-active'"
                style="font-family: 'Outfit', sans-serif;"
              >
                <span 
                  class="front w-full h-16 px-8 rounded-lg border-2 border-[rgba(0,0,0,0.35)] font-bold text-xl flex items-center justify-center gap-3"
                  :style="isLoading ? 'background-color: #543c55; color: white;' : 'background: linear-gradient(135deg, #E99682 0%, #EB9182 33%, #E88592 66%, #E883AE 100%); color: white;'"
                >
                  <svg v-if="!isLoading" class="w-7 h-7 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2.5">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M11 16l-4-4m0 0l4-4m-4 4h14m-5 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h7a3 3 0 013 3v1"></path>
                  </svg>
                  <svg v-else class="animate-spin h-7 w-7 flex-shrink-0" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                  </svg>
                  <span class="flex-shrink-0">{{ isLoading ? 'Initializing...' : 'Sign in with Hackatime' }}</span>
                </span>
              </button>
              
              <div v-if="isDevMode" class="mt-8 pt-8 border-t border-white/20">
                <p class="text-white/60 text-sm mb-3" style="font-family: 'Outfit', sans-serif;">
                  Developer Mode: Paste token directly
                </p>
                <div class="flex gap-2">
                  <input 
                    v-model="directToken"
                    type="text" 
                    placeholder="Paste token..."
                    class="flex-1 p-3 bg-[#2A1F2B] border border-white/20 rounded-lg text-white font-mono text-sm focus:outline-none focus:border-[#E99682] transition-colors"
                    @keyup.enter="handleDirectAuth"
                  />
                  <button 
                    @click="handleDirectAuth"
                    :disabled="!directToken.trim() || isLoading"
                    class="px-4 py-3 rounded-lg border-2 border-[rgba(0,0,0,0.35)] font-bold text-sm transition-all"
                    :class="!directToken.trim() || isLoading ? 'bg-gray-600 text-white/50 cursor-not-allowed' : 'bg-[#E99682] text-white hover:bg-[#d88672]'"
                    style="font-family: 'Outfit', sans-serif;"
                  >
                    Go
                  </button>
                </div>
              </div>
            </div>

            <div v-else class="text-center py-4">
              <div class="mb-8 loader-container">
                <RandomLoader />
              </div>

              <h2 class="text-[28px] font-bold text-white mb-5" style="font-family: 'Outfit', sans-serif;">
                Opening in your browser
              </h2>
              <p class="text-white/70 text-[16px]" style="font-family: 'Outfit', sans-serif;">
                Complete authentication in the browser window that just opened
              </p>

              <button 
                @click="handleManualOpen"
                class="pushable w-full mt-10 mb-6"
                style="font-family: 'Outfit', sans-serif; background-color: #2A1F2B;"
              >
                <span 
                  class="front w-full h-14 px-6 rounded-lg border-2 border-[rgba(0,0,0,0.35)] font-medium text-base flex items-center justify-center"
                  style="background-color: #543c55; color: white;"
                >
                  Didn't work? Click here to open manually
                </span>
              </button>

              <!-- Linux-specific OAuth URL copy section -->
              <div v-if="currentOs === 'linux' && oauthUrl" class="mt-6 mb-6 p-4 bg-[#2A1F2B] border border-white/20 rounded-lg">
                <p class="text-white/70 text-sm mb-3" style="font-family: 'Outfit', sans-serif;">
                  <strong>Linux:</strong> Copy the link to open in your browser manually
                </p>
                <div class="flex gap-2">
                  <input 
                    :value="oauthUrl"
                    readonly
                    class="flex-1 p-3 bg-[#3D2C3E] border border-white/20 rounded-lg text-white font-mono text-xs focus:outline-none focus:border-[#E99682] transition-colors select-all"
                    @click="($event.target as HTMLInputElement)?.select()"
                  />
                  <button 
                    @click="copyOAuthUrl"
                    class="px-4 py-3 rounded-lg border-2 border-[rgba(0,0,0,0.35)] font-bold text-sm transition-all bg-[#E99682] text-white hover:bg-[#d88672]"
                    style="font-family: 'Outfit', sans-serif;"
                  >
                    Copy
                  </button>
                </div>
              </div>

              <button 
                @click="cancelAuth"
                class="text-white/60 text-base hover:text-white transition-colors font-medium"
                style="font-family: 'Outfit', sans-serif;"
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import RandomLoader from '../components/RandomLoader.vue';

const emit = defineEmits<{
  authenticate: [];
  handleDirectOAuthAuth: [token: string];
  openOAuthUrlManually: [];
}>();

const authInProgress = ref(false);
const directToken = ref('');

async function handleLogin() {
  emit('authenticate');
  setTimeout(() => {
    authInProgress.value = true;
  }, 500);
}

function handleManualOpen() {
  emit('openOAuthUrlManually');
}

function cancelAuth() {
  authInProgress.value = false;
}

function handleDirectAuth() {
  if (directToken.value.trim()) {
    emit('handleDirectOAuthAuth', directToken.value.trim());
    directToken.value = '';
  }
}

const props = defineProps<{
  isLoading: boolean;
  isDevMode: boolean;
  oauthUrl: string | null;
  currentOs: string | null;
}>();

async function copyOAuthUrl() {
  if (!props.oauthUrl) return;
  
  try {
    await navigator.clipboard.writeText(props.oauthUrl);
    alert("OAuth URL copied to clipboard!");
  } catch (error) {
    console.error("Failed to copy OAuth URL:", error);
    alert("Failed to copy OAuth URL to clipboard");
  }
}
</script>

<style scoped>
.card-3d {
  position: relative;
  border-radius: 12px;
  padding: 0;
}

.card-3d::before {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: 12px;
  background-color: #2A1F2B;
  z-index: 0;
}

.card-3d-front {
  position: relative;
  transform: translateY(-8px);
  z-index: 1;
}

.pushable {
  border-radius: 12px;
  border: none;
  padding: 0;
  cursor: pointer;
  outline-offset: 4px;
  position: relative;
}

.pushable-active {
  background: linear-gradient(135deg, #B85E6D 0%, #B85E6D 33%, #B5546F 66%, #B55389 100%);
}

.pushable:not(.pushable-active) {
  background-color: #2A1F2B;
}

.pushable:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

.front {
  display: flex;
  align-items: center;
  border-radius: 12px;
  transform: translateY(-6px);
  transition: transform 0.1s ease;
  position: relative;
}

.pushable:active:not(:disabled) .front {
  transform: translateY(-2px);
}

/* Bounce Animation */
@keyframes bounce {
  0%, 100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-10px);
  }
}

.animate-bounce {
  animation: bounce 1s infinite;
}

.loader-container {
  height: 120px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.loader-container :deep(.flex) {
  height: 120px !important;
}
</style>

