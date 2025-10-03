<template>
  <!-- Authentication Section -->
  <div v-if="!authState.is_authenticated" class="flex items-center justify-center min-h-96">
    <div class="text-center max-w-md">
      <h3 class="text-2xl mb-4 text-text-primary">Welcome to Hackatime</h3>
      <p class="text-text-secondary mb-8 leading-relaxed">Connect to your KubeTime account to start tracking your coding time.</p>

      <!-- Production authentication (deep link) -->
      <template v-if="!isDevMode">
        <button 
          @click="authenticate" 
          :disabled="isLoading"
          class="bg-accent-primary text-white border-0 px-8 py-4 rounded-xl text-base font-medium cursor-pointer transition-all duration-200 my-4 w-full hover:bg-accent-secondary hover:shadow-card-hover disabled:bg-text-muted disabled:cursor-not-allowed disabled:transform-none"
        >
          {{ isLoading ? 'Opening Login...' : 'Login with KubeTime' }}
        </button>
        <p class="text-text-secondary text-sm mt-2">This will open your browser for OAuth authentication.</p>
      </template>
      
      <!-- Development authentication options -->
      <template v-else>
        <div class="browser-auth-section">
          <button 
            @click="authenticate" 
            :disabled="isLoading"
            class="bg-accent-primary text-white border-0 px-8 py-4 rounded-xl text-base font-medium cursor-pointer transition-all duration-200 my-4 w-full hover:bg-accent-secondary hover:shadow-card-hover disabled:bg-text-muted disabled:cursor-not-allowed disabled:transform-none"
          >
            {{ isLoading ? 'Opening Login...' : 'Open Browser Login' }}
          </button>
          <p class="text-text-secondary text-sm mt-2">This will open your browser for OAuth authentication.</p>
        </div>
        
        <div class="mt-8 pt-8 border-t border-border-primary text-left">
          <h4 class="text-text-primary mb-2 text-lg">Or paste your token directly</h4>
          <p class="text-text-secondary mb-4 text-sm">Paste your token from the browser callback URL:</p>
          <div class="flex gap-3 mb-4">
            <input 
              :value="directOAuthToken"
              @input="$emit('update:directOAuthToken', ($event.target as HTMLInputElement).value)"
              type="text" 
              placeholder="Paste your token here..."
              class="flex-1 p-3 bg-bg-secondary border border-border-secondary rounded-xl text-text-primary font-mono text-sm focus:outline-none focus:border-accent-primary focus:shadow-[0_0_0_2px_rgba(200,57,79,0.2)]"
              @keyup.enter="handleDirectOAuthAuth"
            />
            <button 
              @click="handleDirectOAuthAuth"
              :disabled="isLoading || !directOAuthToken.trim()"
              class="bg-accent-primary text-white border-0 px-6 py-3 rounded-xl text-sm font-medium cursor-pointer whitespace-nowrap transition-all duration-200 hover:bg-accent-secondary disabled:bg-text-muted disabled:cursor-not-allowed"
            >
              {{ isLoading ? 'Authenticating...' : 'Authenticate' }}
            </button>
          </div>
        </div>
      </template>
    </div>
  </div>

  <!-- Fallback when authenticated but no user stats available -->
  <div v-else-if="!userStats" class="mb-8">
    <div class="bg-bg-secondary p-6 rounded-xl border border-border-primary text-center">
      <h4 class="text-text-primary mb-2 text-lg">No Stats Available</h4>
      <p class="text-text-secondary">Start coding to see your statistics here!</p>
    </div>
  </div>
</template>

<script setup lang="ts">
interface AuthState {
  is_authenticated: boolean;
  access_token: string | null;
  user_info: Record<string, any> | null;
}

interface ApiConfig {
  base_url: string;
}

defineProps<{
  authState: AuthState;
  apiConfig: ApiConfig;
  userData: any;
  userStats: any;
  isLoading: boolean;
  isDevMode: boolean;
  directOAuthToken: string;
}>();

const emit = defineEmits<{
  authenticate: [];
  handleDirectOAuthAuth: [];
  'update:directOAuthToken': [value: string];
}>();

async function authenticate() {
  emit('authenticate');
}

async function handleDirectOAuthAuth() {
  emit('handleDirectOAuthAuth');
}

</script>

<!-- All styles now handled by Tailwind CSS -->