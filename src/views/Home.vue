<template>
  <!-- Authentication Section -->
  <div v-if="!authState.is_authenticated" class="flex items-center justify-center min-h-96">
    <div class="text-center max-w-md">
      <h3 class="text-2xl mb-4 text-text-primary">Welcome to Hackatime</h3>
      <p class="text-text-secondary mb-8 leading-relaxed">Connect to your Hackatime account to start tracking your coding time.</p>

      <!-- Production authentication (deep link) -->
      <template v-if="!isDevMode">
        <button 
          @click="authenticate" 
          :disabled="isLoading"
          class="bg-accent-primary text-white border-0 px-8 py-4 rounded-xl text-base font-medium cursor-pointer transition-all duration-200 my-4 w-full hover:bg-accent-secondary hover:shadow-card-hover disabled:bg-text-muted disabled:cursor-not-allowed disabled:transform-none"
        >
          {{ isLoading ? 'Opening Login...' : 'Login with Hackatime' }}
        </button>
        <p class="text-text-secondary text-sm mt-2">This will open your browser for OAuth authentication.</p>
      
        <button 
          v-if="oauthUrl && !isLoading"
          @click="openOAuthUrlManually" 
          class="bg-bg-secondary text-text-primary border border-border-secondary px-6 py-3 rounded-xl text-sm font-medium cursor-pointer transition-all duration-200 mt-4 w-full hover:bg-bg-tertiary hover:border-accent-primary"
        >
          Link didn't open? Click here
        </button>
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
          
          <button 
            v-if="oauthUrl && !isLoading"
            @click="openOAuthUrlManually" 
            class="bg-bg-secondary text-text-primary border border-border-secondary px-6 py-3 rounded-xl text-sm font-medium cursor-pointer transition-all duration-200 mt-4 w-full hover:bg-bg-tertiary hover:border-accent-primary"
          >
            Link didn't open? Click here
          </button>
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

  <!-- Authenticated Content -->
  <div v-else-if="userStats" class="flex flex-col h-full min-h-0">
    <!-- Welcome Header -->
    <div class="mb-6">
      <h1 class="text-[40px] sm:text-[32px] lg:text-[40px] font-bold italic text-white m-0 mb-2" style="font-family: 'Outfit', sans-serif;">
        welcome back, {{ userData?.emails?.[0]?.split('@')[0] || 'user' }}
      </h1>
      <p class="text-[20px] sm:text-[16px] lg:text-[20px] text-white m-0" style="font-family: 'Outfit', sans-serif;">
        {{ motd }}
      </p>
    </div>

    <!-- Streak Card -->
    <div class="card-3d mb-6 flex-shrink-0">
      <div class="relative rounded-[8px] overflow-hidden border-2 border-black card-3d-front" style="background: linear-gradient(135deg, #E99682 0%, #EB9182 33%, #E88592 66%, #E883AE 100%);">
        <div class="flex items-center p-4 relative z-10 flex-wrap gap-4">
        <!-- Flame icon with streak -->
        <div class="relative">
          <img src="/flame-icon.svg" alt="Streak" class="w-16 h-16" />
          <div class="absolute inset-0 flex items-end justify-center pb-1.5">
            <div class="text-white drop-shadow-lg font-bold" :class="{
              'text-3xl': (userStats.current_streak || 0) < 10,
              'text-2xl': (userStats.current_streak || 0) >= 10 && (userStats.current_streak || 0) < 100,
              'text-xl': (userStats.current_streak || 0) >= 100 && (userStats.current_streak || 0) < 1000,
              'text-lg': (userStats.current_streak || 0) >= 1000
            }">
              {{ userStats.current_streak || 0 }}
            </div>
          </div>
        </div>

        <!-- Text -->
        <div class="flex-1 min-w-0">
          <p class="text-white text-[13px] m-0" style="font-family: 'Outfit', sans-serif;">you have a</p>
          <p class="text-white text-[26px] font-bold m-0 leading-tight" style="font-family: 'Outfit', sans-serif;">
            {{ userStats.current_streak || 0 }} days streak
          </p>
        </div>

        <!-- Hours Coded Box -->
        <div class="backdrop-blur-[2px] bg-[rgba(166,82,14,0.5)] border-2 border-[rgba(166,82,14,0.35)] rounded-[4px] h-[65px] w-[100px] flex flex-col items-center justify-center flex-shrink-0">
          <p class="text-white text-[32px] font-bold m-0 leading-none" style="font-family: 'Outfit', sans-serif;">
            {{ Math.round((userStats.weekly_stats?.time_coded_seconds || 0) / 3600) }}
          </p>
          <p class="text-white text-[10px] font-bold m-0 mt-1 px-1 text-center leading-tight" style="font-family: 'Outfit', sans-serif;">
            HOURS CODED
          </p>
        </div>

        <!-- Rank Box -->
        <div class="backdrop-blur-[2px] bg-[rgba(166,82,14,0.5)] border-2 border-[rgba(166,82,14,0.35)] rounded-[4px] h-[65px] w-[100px] flex flex-col items-center justify-center flex-shrink-0">
          <p class="text-white text-[32px] font-bold m-0 leading-none" style="font-family: 'Outfit', sans-serif;">
            #1
          </p>
          <p class="text-white text-[10px] font-bold m-0 mt-1 px-1 text-center leading-tight" style="font-family: 'Outfit', sans-serif;">
            AMONG FRIENDS
          </p>
        </div>
      </div>
    </div>
    </div>

    <!-- Stats Section -->
    <div class="card-3d card-3d-stats flex-1 min-h-0">
      <div class="rounded-[8px] border border-black p-6 card-3d-front h-full flex flex-col" style="background-color: #3D2C3E;">
        
        
        <!-- Weekly Coding Time Card -->
        <div class="rounded-lg px-4 py-2 mb-4 border-2" :style="getWeeklyCardStyle(userStats?.calculated_metrics?.weekly_change_percent || 0)">
          <div class="flex items-center gap-4">
            <div class="flex items-center gap-3">
              <div class="text-white text-[28px] font-bold leading-none" style="font-family: 'Outfit', sans-serif;">
                {{ (userStats?.calculated_metrics?.weekly_change_percent || 0) > 0 ? '+' : '' }}{{ (userStats?.calculated_metrics?.weekly_change_percent || 0).toFixed(0) }}%
              </div>
              <p class="text-white text-[14px] font-semibold m-0 opacity-95 tracking-wide" style="font-family: 'Outfit', sans-serif;">
                Weekly Coding Time vs last week
              </p>
            </div>
            <div class="ml-auto">
              <p class="text-white text-[12px] m-0 opacity-85 text-right" style="font-family: 'Outfit', sans-serif;">
                {{ (userStats?.calculated_metrics?.weekly_hours || 0).toFixed(1) }}h
              </p>
            </div>
          </div>
        </div>

        <!-- Weekly Chart -->
        <div class="bg-[rgba(50,36,51,0.15)] border-2 border-[rgba(50,36,51,0.25)] rounded-lg p-5 mt-2 flex-1 flex flex-col min-h-0">
          <p class="text-white text-[12px] m-0 mb-4 opacity-80" style="font-family: 'Outfit', sans-serif; letter-spacing: 0.2px;">
            Last 7 Days Activity
          </p>
          <div class="mt-1 flex-1 min-h-0">
            <WeeklyChart :data="weeklyChartData" />
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- Fallback when authenticated but no user stats available -->
  <div v-else class="flex items-center justify-center min-h-96">
    <RandomLoader />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import WeeklyChart from '../components/WeeklyChart.vue';
import RandomLoader from '../components/RandomLoader.vue';
import motdData from '../motd.json';

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
  weeklyChartData: Array<{
    date: string;
    day_name: string;
    hours: number;
    percentage: number;
  }>;
  isLoading: boolean;
  isDevMode: boolean;
  directOAuthToken: string;
  oauthUrl: string | null;
}>();

const emit = defineEmits<{
  authenticate: [];
  handleDirectOAuthAuth: [];
  openOAuthUrlManually: [];
  'update:directOAuthToken': [value: string];
}>();

const motd = ref<string>('');

onMounted(() => {
  motd.value = getMotd();
});

function getMotd(): string {
  const today = new Date();
  const month = String(today.getMonth() + 1).padStart(2, '0');
  const day = String(today.getDate()).padStart(2, '0');
  const dateKey = `${month}-${day}`;

  
  const holidayData = (motdData.holidays as Record<string, { name: string; messages: string[] }>)[dateKey];
  
  if (holidayData && holidayData.messages.length > 0) {
    
    return holidayData.messages[Math.floor(Math.random() * holidayData.messages.length)];
  }

  
  return motdData.regular[Math.floor(Math.random() * motdData.regular.length)];
}

async function authenticate() {
  emit('authenticate');
}

async function handleDirectOAuthAuth() {
  emit('handleDirectOAuthAuth');
}

async function openOAuthUrlManually() {
  emit('openOAuthUrlManually');
}

function getWeeklyCardStyle(percentage: number): string {
  const positiveColor = { r: 52, g: 148, b: 230 };   
  const negativeColor = { r: 236, g: 110, b: 173 }; 
  
  const intensity = Math.min(Math.abs(percentage) / 100, 1);
  
  let color;
  if (percentage >= 0) {
    color = positiveColor;
  } else {
    const t = intensity;
    color = {
      r: Math.round(positiveColor.r + (negativeColor.r - positiveColor.r) * t),
      g: Math.round(positiveColor.g + (negativeColor.g - positiveColor.g) * t),
      b: Math.round(positiveColor.b + (negativeColor.b - positiveColor.b) * t)
    };
  }
  
  return `background-color: rgb(${color.r}, ${color.g}, ${color.b}); border-color: rgba(0,0,0,0.25);`;
}

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
  background: linear-gradient(135deg, #B85E6D 0%, #B85E6D 33%, #B5546F 66%, #B55389 100%);
  z-index: 0;
}

.card-3d-stats::before {
  background: #2A1F2B !important;
}

.card-3d-front {
  position: relative;
  transform: translateY(-6px);
  z-index: 1;
}
</style>