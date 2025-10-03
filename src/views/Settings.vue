<template>
  <div class="min-h-72">
    <div class="space-y-8">
      <!-- Theme Settings -->
      <div class="space-y-4">
        <h3 class="text-lg font-semibold text-text-primary">Appearance</h3>
        <div class="bg-bg-secondary border border-border-primary rounded-xl p-6">
          <div class="flex items-center justify-between">
            <div>
              <h4 class="font-medium text-text-primary mb-1">Theme</h4>
              <p class="text-sm text-text-secondary">Choose between dark and light mode</p>
            </div>
            <div class="flex items-center gap-3">
              <span class="text-sm text-text-secondary">{{ currentTheme === 'dark' ? 'Dark' : 'Light' }}</span>
              <button
                @click="toggleTheme"
                class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-accent-primary focus:ring-offset-2"
                :class="currentTheme === 'dark' ? 'bg-accent-primary' : 'bg-border-primary'"
              >
                <span
                  class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform"
                  :class="currentTheme === 'dark' ? 'translate-x-6' : 'translate-x-1'"
                />
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- App Information -->
      <div class="space-y-4">
        <h3 class="text-lg font-semibold text-text-primary">About</h3>
        <div class="bg-bg-secondary border border-border-primary rounded-xl p-6">
          <div class="space-y-3">
            <div class="flex justify-between">
              <span class="text-text-secondary">Version</span>
              <span class="text-text-primary font-medium">1.0.0</span>
            </div>
            <div class="flex justify-between">
              <span class="text-text-secondary">Build</span>
              <span class="text-text-primary font-medium">Development</span>
            </div>
            <div class="flex justify-between items-center">
              <span class="text-text-secondary">Updates</span>
              <div class="flex items-center gap-2">
                <span v-if="updateStatus === 'checking'" class="text-text-secondary text-sm">Checking...</span>
                <span v-else-if="updateStatus === 'available'" class="text-accent-primary text-sm">Update available</span>
                <span v-else-if="updateStatus === 'latest'" class="text-green-500 text-sm">Up to date</span>
                <span v-else-if="updateStatus === 'error'" class="text-red-500 text-sm">Check failed</span>
                <button
                  @click="checkForUpdates"
                  :disabled="updateStatus === 'checking'"
                  class="px-3 py-1 text-xs bg-accent-primary text-white rounded-lg hover:bg-orange-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                >
                  {{ updateStatus === 'checking' ? 'Checking...' : 'Check for Updates' }}
                </button>
              </div>
            </div>
            <div v-if="updateInfo" class="mt-4 p-4 bg-bg-tertiary rounded-lg border border-border-secondary">
              <div class="space-y-2">
                <div class="flex justify-between">
                  <span class="text-text-secondary text-sm">New Version</span>
                  <span class="text-text-primary text-sm font-medium">{{ updateInfo.version }}</span>
                </div>
                <div v-if="updateInfo.notes" class="text-text-secondary text-sm">
                  <p class="font-medium mb-1">Release Notes:</p>
                  <p class="whitespace-pre-wrap">{{ updateInfo.notes }}</p>
                </div>
                <div class="flex gap-2 mt-3">
                  <button
                    @click="downloadAndInstallUpdate"
                    :disabled="isInstallingUpdate"
                    class="px-4 py-2 bg-accent-primary text-white rounded-lg hover:bg-orange-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors text-sm"
                  >
                    {{ isInstallingUpdate ? 'Installing...' : 'Install Update' }}
                  </button>
                  <button
                    @click="updateInfo = null"
                    class="px-4 py-2 bg-bg-primary border border-border-secondary text-text-primary rounded-lg hover:bg-bg-secondary transition-colors text-sm"
                  >
                    Later
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- API Key -->
      <div v-if="apiKey" class="space-y-4">
        <h3 class="text-lg font-semibold text-text-primary">API Access</h3>
        <div class="bg-bg-secondary border border-border-primary rounded-xl p-6">
          <div class="space-y-4">
            <div>
              <h4 class="font-medium text-text-primary mb-2">Your API Key</h4>
              <p class="text-sm text-text-secondary mb-4">Use this key to authenticate with the KubeTime API</p>
              <div class="flex gap-3 items-center">
                <input 
                  :type="showApiKey ? 'text' : 'password'" 
                  :value="apiKey" 
                  readonly 
                  class="flex-1 p-3 bg-bg-tertiary border border-border-secondary rounded-xl text-text-primary font-mono text-sm"
                />
                <div class="flex gap-2">
                  <button @click="$emit('update:showApiKey', !showApiKey)" class="p-3 border border-border-secondary rounded-xl cursor-pointer text-sm min-w-11 transition-all duration-200 bg-bg-tertiary text-text-secondary hover:bg-bg-primary hover:text-text-primary hover:border-accent-primary">
                    <svg v-if="showApiKey" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.878 9.878L8.464 8.464M9.878 9.878L12 12m-2.122-2.122l1.415 1.415M12 12l2.122 2.122m-2.122-2.122L12 12m2.122 2.122l-1.415-1.415M12 12l-2.122-2.122"></path>
                    </svg>
                    <svg v-else class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"></path>
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"></path>
                    </svg>
                  </button>
                  <button @click="copyApiKey" class="p-3 border border-accent-info rounded-xl cursor-pointer text-sm min-w-11 transition-all duration-200 bg-accent-info text-white hover:bg-blue-400">
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"></path>
                    </svg>
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Preferences -->
      <div class="space-y-4">
        <h3 class="text-lg font-semibold text-text-primary">Preferences</h3>
        <div class="bg-bg-secondary border border-border-primary rounded-xl p-6">
          <div class="space-y-4">
            <div class="flex items-center justify-between">
              <div>
                <h4 class="font-medium text-text-primary mb-1">Auto-start</h4>
                <p class="text-sm text-text-secondary">Start with system</p>
              </div>
              <button
                class="relative inline-flex h-6 w-11 items-center rounded-full bg-border-primary transition-colors focus:outline-none focus:ring-2 focus:ring-accent-primary focus:ring-offset-2"
              >
                <span class="inline-block h-4 w-4 transform rounded-full bg-white translate-x-1 transition-transform" />
              </button>
            </div>
            <div class="flex items-center justify-between">
              <div>
                <h4 class="font-medium text-text-primary mb-1">Discord RPC</h4>
                <p class="text-sm text-text-secondary">Show coding activity in Discord</p>
              </div>
              <button
                @click="toggleDiscordRpc"
                :disabled="isLoading"
                class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-accent-primary focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed"
                :class="discordRpcEnabled ? 'bg-accent-primary' : 'bg-border-primary'"
              >
                <span
                  v-if="isLoading"
                  class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform animate-pulse"
                  :class="discordRpcEnabled ? 'translate-x-6' : 'translate-x-1'"
                />
                <span
                  v-else
                  class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform"
                  :class="discordRpcEnabled ? 'translate-x-6' : 'translate-x-1'"
                />
              </button>
            </div>
            <div class="flex items-center justify-between">
              <div>
                <h4 class="font-medium text-text-primary mb-1">Notifications</h4>
                <p class="text-sm text-text-secondary">Show desktop notifications</p>
              </div>
              <button
                class="relative inline-flex h-6 w-11 items-center rounded-full bg-accent-primary transition-colors focus:outline-none focus:ring-2 focus:ring-accent-primary focus:ring-offset-2"
              >
                <span class="inline-block h-4 w-4 transform rounded-full bg-white translate-x-6 transition-transform" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import type { Theme } from '../composables/useTheme'

defineProps<{
  currentTheme: Theme
  toggleTheme: () => void
  apiKey: string | null
  showApiKey: boolean
}>()

const emit = defineEmits<{
  copyApiKey: []
  'update:showApiKey': [value: boolean]
}>()

const discordRpcEnabled = ref(false);
const isLoading = ref(false);

// Update functionality
const updateStatus = ref<'idle' | 'checking' | 'available' | 'latest' | 'error'>('idle');
const updateInfo = ref<{
  version: string;
  notes?: string;
  date?: string;
} | null>(null);
const isInstallingUpdate = ref(false);

async function loadDiscordRpcState() {
  try {
    discordRpcEnabled.value = await invoke("get_discord_rpc_enabled");
  } catch (error) {
    console.error("Failed to load Discord RPC state:", error);
  }
}

async function toggleDiscordRpc() {
  if (isLoading.value) return;
  
  isLoading.value = true;
  try {
    const newState = !discordRpcEnabled.value;
    await invoke("set_discord_rpc_enabled", { enabled: newState });
    discordRpcEnabled.value = newState;
  } catch (error) {
    console.error("Failed to toggle Discord RPC:", error);
    // Revert the UI state on error
    discordRpcEnabled.value = !discordRpcEnabled.value;
  } finally {
    isLoading.value = false;
  }
}

function copyApiKey() {
  emit('copyApiKey')
}

// Update functions
async function checkForUpdates() {
  if (updateStatus.value === 'checking') return;
  
  updateStatus.value = 'checking';
  updateInfo.value = null;
  
  try {
    const update = await check();
    
    if (update) {
      updateStatus.value = 'available';
      updateInfo.value = {
        version: update.version,
        notes: update.body,
        date: update.date
      };
    } else {
      updateStatus.value = 'latest';
    }
  } catch (error) {
    console.error('Failed to check for updates:', error);
    updateStatus.value = 'error';
  }
}

async function downloadAndInstallUpdate() {
  if (!updateInfo.value || isInstallingUpdate.value) return;
  
  isInstallingUpdate.value = true;
  
  try {
    const update = await check();
    if (update) {
      let downloaded = 0;
      let contentLength = 0;
      
      await update.downloadAndInstall((event) => {
        switch (event.event) {
          case 'Started':
            contentLength = event.data.contentLength ?? 0;
            console.log(`Started downloading ${event.data.contentLength} bytes`);
            break;
          case 'Progress':
            downloaded += event.data.chunkLength;
            console.log(`Downloaded ${downloaded} from ${contentLength}`);
            break;
          case 'Finished':
            console.log('Download finished');
            break;
        }
      });
      
      console.log('Update installed, restarting...');
      await relaunch();
    }
  } catch (error) {
    console.error('Failed to install update:', error);
    isInstallingUpdate.value = false;
  }
}

// Load Discord RPC state on mount
onMounted(() => {
  loadDiscordRpcState();
});
</script>

<!-- All styles now handled by Tailwind CSS -->