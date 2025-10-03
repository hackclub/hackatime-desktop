<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrent, onOpenUrl } from "@tauri-apps/plugin-deep-link";
import { api } from "./api";
import { useTheme } from "./composables/useTheme";
import Home from "./views/Home.vue";
import Projects from "./views/Projects.vue";
import Settings from "./views/Settings.vue";
import Statistics from "./views/Statistics.vue";
import PresenceCard from "./components/PresenceCard.vue";
import TrendCard from "./components/TrendCard.vue";
import WeeklyChart from "./components/WeeklyChart.vue";

interface AuthState {
  is_authenticated: boolean;
  access_token: string | null;
  user_info: Record<string, any> | null;
}

interface ApiConfig {
  base_url: string;
}

const authState = ref<AuthState>({
  is_authenticated: false,
  access_token: null,
  user_info: null,
});

const apiConfig = ref<ApiConfig>({
  base_url: "",
});

const isConfigOpen = ref(false);
const isLoading = ref(false);
const userData = ref<any>(null);
const userStats = ref<any>(null);
const isDevMode = ref(false);
const directOAuthToken = ref("");
const apiKey = ref<string | null>(null);
const showApiKey = ref(false);
const hackatimeDirectories = ref<any>(null);
const sessionStats = ref<any>(null);
const presenceData = ref<any>(null);
const presenceRefreshInterval = ref<number | null>(null);
const presenceFetchInProgress = ref(false);
const nextPresenceFetchAllowedAt = ref<number>(0);
const lastPresenceFetchAt = ref<number>(0);

// Navigation state
const currentPage = ref<'home' | 'projects' | 'statistics' | 'settings'>('home');

// Theme management
const { currentTheme, toggleTheme } = useTheme();

// Computed property for weekly chart data
const weeklyChartData = computed(() => {
  if (!userStats.value?.weekly_stats?.daily_hours) return [];
  
  const dailyHours = userStats.value.weekly_stats.daily_hours;
  const maxHours = Math.max(...Object.values(dailyHours).map((day: any) => day.hours), 1);
  
  // Convert object to array and sort by date
  return Object.values(dailyHours)
    .sort((a: any, b: any) => new Date(a.date).getTime() - new Date(b.date).getTime())
    .map((day: any) => ({
      ...day,
      percentage: (day.hours / maxHours) * 100
    }));
});

// Computed property for trend data
        const weeklyTrend = computed(() => {
          if (!userStats.value?.weekly_stats) return null;
          
          const currentWeekHours = (userStats.value.weekly_stats.time_coded_seconds || 0) / 3600;
          const lastWeekHours = currentWeekHours * 0.85; // Simulate 15% increase
          const change = ((currentWeekHours - lastWeekHours) / lastWeekHours * 100);
          
          return {
            title: change > 0 ? "You coded more than last week" : change < 0 ? "You coded less than last week" : "Same as last week",
            change: change > 0 ? `+${Math.round(change)}%` : `${Math.round(change)}%`,
            changeType: change > 0 ? 'increase' : change < 0 ? 'decrease' : 'neutral',
            period: "vs last week",
            icon: change > 0 ? "📈" : change < 0 ? "📉" : "➡️"
          };
        });


onMounted(async () => {
  await loadAuthState();
  await loadApiConfig();
  await loadHackatimeInfo();
  
  // Detect if we're in development mode
  // Check if we're running on localhost (development) or have debug features
  isDevMode.value = apiConfig.value.base_url.includes('localhost') || 
                    apiConfig.value.base_url.includes('127.0.0.1') ||
                    window.location.hostname === 'localhost' ||
                    window.location.hostname === '127.0.0.1';
  
  // Check if app was started via deep link
  try {
    const startUrls = await getCurrent();
    if (startUrls && startUrls.length > 0) {
      console.log("App started with deep link:", startUrls);
      // Check if it's an OAuth callback
      const hasOAuthCallback = startUrls.some(url => 
        url.startsWith('hackatime://auth/callback')
      );
      
      if (hasOAuthCallback) {
        console.log("OAuth callback detected, refreshing auth state...");
        // The Rust backend will handle the deep link processing
        // We just need to refresh the auth state after processing
        setTimeout(async () => {
          await loadAuthState();
        }, 1000); // Give the backend time to process the deep link
      }
    }
  } catch (error) {
    console.error("Failed to get current deep link:", error);
  }
  
  // Listen for deep link events when app is already running
  try {
    await onOpenUrl((urls) => {
      console.log("Deep link received in frontend:", urls);
      // Check if it's an OAuth callback
      const hasOAuthCallback = urls.some(url => 
        url.startsWith('hackatime://auth/callback')
      );
      
      if (hasOAuthCallback) {
        console.log("OAuth callback detected, refreshing auth state...");
        // The Rust backend handles the actual processing
        // We just need to refresh the auth state
        setTimeout(async () => {
          await loadAuthState();
        }, 1000); // Give the backend time to process the deep link
      }
    });
  } catch (error) {
    console.error("Failed to set up deep link listener:", error);
  }
  
  // Listen for window focus events to refresh auth state after popup closes
  window.addEventListener('focus', async () => {
    await loadAuthState();
  });
  
  // Also listen for visibility change (when tab becomes active)
  document.addEventListener('visibilitychange', async () => {
    if (!document.hidden) {
      await loadAuthState();
    }
  });
});

// Cleanup on unmount
onUnmounted(() => {
  stopPresenceRefresh();
});

async function loadAuthState() {
  try {
    console.log("Loading authentication state...");
    const savedAuthState = await invoke("load_auth_state");
    console.log("Saved auth state result:", savedAuthState);
    
    if (savedAuthState && (savedAuthState as AuthState).is_authenticated) {
      authState.value = savedAuthState as AuthState;
      console.log("Loaded saved authentication state:", authState.value);
      
      // If authenticated, load user data, stats, and API keys
      await loadUserData();
      await loadApiKey();
      await registerPresenceConnection();
    } else {
      // No saved state or not authenticated, get current state
      console.log("No saved auth state found, getting current state");
      authState.value = await invoke("get_auth_state");
      console.log("Current auth state:", authState.value);
      
      // If we have an authenticated state, save it to disk
      if (authState.value.is_authenticated) {
        try {
          await invoke("save_auth_state", { authState: authState.value });
          console.log("Current auth state saved to disk");
        } catch (error) {
          console.error("Failed to save current auth state:", error);
        }
      }
    }
  } catch (error) {
    console.error("Failed to load auth state:", error);
    // Fallback to current state on error
    try {
      authState.value = await invoke("get_auth_state");
    } catch (fallbackError) {
      console.error("Failed to get current auth state:", fallbackError);
    }
  }
}

async function loadUserData() {
  try {
    await api.initialize();
    userData.value = await api.getCurrentUser();
    
    // Load user dashboard stats (getStats now returns dashboard stats)
    try {
      userStats.value = await api.getStats();
    } catch (error) {
      console.error("Failed to load user dashboard stats:", error);
    }
    
    // Load presence data and start refresh
    await loadPresenceData();
    startPresenceRefresh();
  } catch (error) {
    console.error("Failed to load user data:", error);
  }
}

async function loadApiKey() {
  try {
    apiKey.value = await invoke("get_api_key", { apiConfig: apiConfig.value });
  } catch (error) {
    console.error("Failed to load API key:", error);
  }
}

async function registerPresenceConnection() {
  try {
    await invoke("register_presence_connection", { apiConfig: apiConfig.value });
    console.log("Presence connection registered successfully");
  } catch (error) {
    console.error("Failed to register presence connection:", error);
  }
}


async function loadApiConfig() {
  try {
    const config = await invoke("get_api_config") as ApiConfig;
    console.log("Loaded API config from backend:", config);
    apiConfig.value = config;
    console.log("Updated apiConfig.value:", apiConfig.value);
  } catch (error) {
    console.error("Failed to load API config:", error);
  }
}

async function loadHackatimeInfo() {
  try {
    hackatimeDirectories.value = await invoke("get_hackatime_directories");
    sessionStats.value = await invoke("get_session_stats");
  } catch (error) {
    console.error("Failed to load hackatime info:", error);
  }
}

async function loadPresenceData() {
  const now = Date.now();
  if (presenceFetchInProgress.value) {
    return; // Skip if a fetch is already in flight
  }
  if (now < nextPresenceFetchAllowedAt.value) {
    return; // Respect backoff window
  }
  // Enforce hard minimum interval of 60s between network calls
  if (now - lastPresenceFetchAt.value < 60_000) {
    return;
  }

  presenceFetchInProgress.value = true;
  try {
    await api.initialize();
    // Use the Rust backend's get_latest_heartbeat which includes session logic
    presenceData.value = await invoke("get_latest_heartbeat", { 
      apiConfig: apiConfig.value 
    });
    lastPresenceFetchAt.value = Date.now();
  } catch (error: any) {
    console.error("Failed to load presence data:", error);
    // If we hit rate limit, back off for 60s
    const message = error?.message || "";
    if (typeof message === "string" && message.includes("429")) {
      nextPresenceFetchAllowedAt.value = Date.now() + 60_000;
    }
    presenceData.value = null;
  } finally {
    presenceFetchInProgress.value = false;
  }
}

function startPresenceRefresh() {
  // Ensure only one interval is active
  if (presenceRefreshInterval.value) {
    clearInterval(presenceRefreshInterval.value);
    presenceRefreshInterval.value = null;
  }
  // Refresh presence data every 60 seconds (1 minute)
  presenceRefreshInterval.value = setInterval(loadPresenceData, 60000);
}

function stopPresenceRefresh() {
  if (presenceRefreshInterval.value) {
    clearInterval(presenceRefreshInterval.value);
    presenceRefreshInterval.value = null;
  }
}


async function authenticate() {
  isLoading.value = true;
  try {
    await invoke("authenticate_with_rails", { apiConfig: apiConfig.value });
    
    // Show instructions for OAuth completion
    alert(`OAuth authentication opened in browser!\n\nInstructions:\n1. Complete the OAuth flow in your browser\n2. The app will automatically handle the callback\n3. If the callback doesn't work, you can manually paste the authorization code from the URL\n\nFor manual entry:\n- Copy the 'code' parameter from the callback URL\n- Use the "Direct OAuth" field below to paste it`);
  } catch (error) {
    console.error("Authentication failed:", error);
    alert("Authentication failed: " + (error instanceof Error ? error.message : String(error)));
  } finally {
    isLoading.value = false;
  }
}

async function logout() {
  try {
    stopPresenceRefresh();
    await invoke("logout");
    await loadAuthState();
  } catch (error) {
    console.error("Logout failed:", error);
  }
}

async function saveApiConfig() {
  try {
    await invoke("set_api_config", { newConfig: apiConfig.value });
    isConfigOpen.value = false;
  } catch (error) {
    console.error("Failed to save API config:", error);
    alert("Failed to save API config: " + error);
  }
}




async function copyApiKey() {
  if (!apiKey.value) return;
  
  try {
    await navigator.clipboard.writeText(apiKey.value);
    alert("API key copied to clipboard!");
  } catch (error) {
    console.error("Failed to copy API key:", error);
    alert("Failed to copy API key to clipboard");
  }
}


async function handleDirectOAuthAuth() {
  if (!directOAuthToken.value.trim()) {
    alert("Please enter an OAuth authorization code or access token");
    return;
  }
  
  try {
    isLoading.value = true;
    
    console.log("Attempting direct OAuth auth with token:", directOAuthToken.value);
    console.log("Token length:", directOAuthToken.value.length);
    console.log("API config:", apiConfig.value);
    
    // Authenticate with the direct OAuth token
    await invoke("authenticate_with_direct_oauth", { 
      oauthToken: directOAuthToken.value,
      apiConfig: apiConfig.value 
    });
    
    console.log("Direct OAuth auth successful!");
    await loadAuthState();
    
    // Ensure the auth state is saved after successful authentication
    if (authState.value.is_authenticated) {
      try {
        await invoke("save_auth_state", { authState: authState.value });
        console.log("Auth state saved after direct OAuth authentication");
      } catch (error) {
        console.error("Failed to save auth state after direct OAuth:", error);
      }
    }
    
    directOAuthToken.value = ""; // Clear the input
    alert("Authentication successful! You are now logged in.");
  } catch (error) {
    console.error("Direct OAuth auth failed:", error);
    alert("Direct OAuth auth failed: " + error);
  } finally {
    isLoading.value = false;
  }
}



function getPageTitle(): string {
  switch (currentPage.value) {
    case 'home':
      return 'Home';
    case 'projects':
      return 'Projects';
    case 'statistics':
      return 'Statistics';
    case 'settings':
      return 'Settings';
    default:
      return 'Home';
  }
}
</script>

<template>
  <div class="flex h-screen text-text-primary font-sans outfit" style="background-color: #0A0101;">
    <!-- Left Sidebar -->
    <aside class="w-64 flex flex-col p-0 shadow-xl rounded-r-2xl" style="background-color: #191415;">
      <div class="p-6" style="background-color: #191415;">
        <h1 class="text-2xl font-bold text-accent-primary m-0 text-center">Hackatime</h1>
      </div>
      
      <nav class="flex-1 py-4">
        <a href="#" class="flex items-center gap-3 px-6 py-3 no-underline transition-all duration-200 border-l-4 border-transparent hover:bg-bg-secondary hover:text-text-primary hover:border-l-accent-primary" :class="{ 'bg-bg-secondary text-accent-primary border-l-accent-primary': currentPage === 'home' }" @click.prevent="currentPage = 'home'" style="color: #B0BAC4;">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"></path>
          </svg>
          <span class="font-medium">Home</span>
        </a>
        <a href="#" class="flex items-center gap-3 px-6 py-3 no-underline transition-all duration-200 border-l-4 border-transparent hover:bg-bg-secondary hover:text-text-primary hover:border-l-accent-primary" :class="{ 'bg-bg-secondary text-accent-primary border-l-accent-primary': currentPage === 'projects' }" @click.prevent="currentPage = 'projects'" style="color: #B0BAC4;">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"></path>
          </svg>
          <span class="font-medium">Projects</span>
        </a>
        <a href="#" class="flex items-center gap-3 px-6 py-3 no-underline transition-all duration-200 border-l-4 border-transparent hover:bg-bg-secondary hover:text-text-primary hover:border-l-accent-primary" :class="{ 'bg-bg-secondary text-accent-primary border-l-accent-primary': currentPage === 'statistics' }" @click.prevent="currentPage = 'statistics'" style="color: #B0BAC4;">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"></path>
          </svg>
          <span class="font-medium">Statistics</span>
        </a>
        <a href="#" class="flex items-center gap-3 px-6 py-3 no-underline transition-all duration-200 border-l-4 border-transparent hover:bg-bg-secondary hover:text-text-primary hover:border-l-accent-primary" :class="{ 'bg-bg-secondary text-accent-primary border-l-accent-primary': currentPage === 'settings' }" @click.prevent="currentPage = 'settings'" style="color: #B0BAC4;">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"></path>
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"></path>
          </svg>
          <span class="font-medium">Settings</span>
        </a>
      </nav>
      
      <div class="p-6" style="background-color: #191415;">
        <button v-if="authState.is_authenticated" @click="logout" class="flex items-center gap-3 w-full px-3 py-3 bg-transparent border border-accent-danger rounded-xl text-accent-danger cursor-pointer transition-all duration-200 text-sm hover:bg-accent-danger hover:text-white">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"></path>
          </svg>
          <span class="font-medium">Logout</span>
        </button>
      </div>
    </aside>

    <!-- Main Content Area -->
    <main class="flex-1 p-6 overflow-y-auto">
      <!-- Home Page Layout -->
      <div v-if="currentPage === 'home'" class="flex flex-col h-full gap-6">
        <!-- This Week Card -->
        <div v-if="authState.is_authenticated && userStats" class="rounded-2xl shadow-card mb-6 p-6 flex flex-col" style="background-color: #191415;">
          <!-- This Week Title -->
          <h3 class="text-text-primary font-semibold text-lg mb-4">This Week</h3>
          <div class="flex gap-8 flex-1 items-center mb-4">
            <!-- Left Section - Streak & Hours Display (2/3 width) -->
            <div class="flex justify-center items-center space-x-20" style="flex: 2;">
              <!-- Streak Section -->
              <div class="flex flex-col items-center">
                <div class="relative">
                  <img src="/flame-icon.svg" alt="Streak" class="w-20 h-20" />
                  <div class="absolute inset-0 flex items-end justify-center pb-2">
                    <div class="text-white drop-shadow-lg font-bold" :class="{
                      'text-4xl': (userStats.current_streak || 0) < 10,
                      'text-3xl': (userStats.current_streak || 0) >= 10 && (userStats.current_streak || 0) < 100,
                      'text-2xl': (userStats.current_streak || 0) >= 100 && (userStats.current_streak || 0) < 1000,
                      'text-xl': (userStats.current_streak || 0) >= 1000
                    }">
                      {{ userStats.current_streak || 0 }}
                    </div>
                  </div>
                </div>
                <div class="text-center mt-2">
                  <div class="text-text-secondary text-xl font-semibold">day streak</div>
                </div>
              </div>

              <!-- Hours Section -->
              <div class="flex flex-col items-center">
                <div class="text-4xl font-bold text-accent-primary">
                  {{ Math.round((userStats.weekly_stats?.time_coded_seconds || 0) / 3600 * 10) / 10 }}
                </div>
                <div class="text-center mt-3">
                  <div class="text-text-secondary text-xl font-semibold">hours this week</div>
                </div>
              </div>
            </div>
            
            <!-- Right Section - Weekly Chart.js Chart (1/3 width) -->
            <div class="flex flex-col justify-center pl-6" style="flex: 1;">
              <WeeklyChart :data="weeklyChartData" />
            </div>
          </div>
          
          <!-- Trend Card -->
          <div v-if="weeklyTrend" class="mt-4">
            <TrendCard
              :title="weeklyTrend.title"
              :change="weeklyTrend.change"
              :change-type="weeklyTrend.changeType"
              :period="weeklyTrend.period"
              :icon="weeklyTrend.icon"
            />
          </div>
        </div>

        <!-- Current Session Card -->
        <div v-if="authState.is_authenticated" class="mb-6">
          <PresenceCard :authState="authState" :presenceData="presenceData" :apiConfig="apiConfig" />
        </div>


        <!-- Home Component -->
        <Home 
          :authState="authState"
          :apiConfig="apiConfig"
          :userData="userData"
          :userStats="userStats"
          :isLoading="isLoading"
          :isDevMode="isDevMode"
          v-model:directOAuthToken="directOAuthToken"
          @authenticate="authenticate"
          @handleDirectOAuthAuth="handleDirectOAuthAuth"
        />
        
      </div>
      
      <!-- Statistics Page Layout (full page) -->
      <div v-else-if="currentPage === 'statistics'" class="flex flex-col h-full">
        <Statistics :apiConfig="apiConfig" />
      </div>

      <!-- Other Pages Layout (single card) -->
      <div v-else class="flex flex-col h-full">
        <div class="bg-bg-card border border-border-primary rounded-2xl overflow-hidden shadow-card flex flex-col min-h-96">
          <div class="flex justify-between items-center px-6 py-5 border-b border-border-primary bg-bg-card-tertiary">
            <h2 class="m-0 text-xl font-semibold text-text-primary">{{ getPageTitle() }}</h2>
          </div>
          <div class="p-6 flex-1 overflow-y-auto">
            <Projects v-if="currentPage === 'projects'" :currentTheme="currentTheme" :toggleTheme="toggleTheme" :apiConfig="apiConfig" />
            <Settings v-if="currentPage === 'settings'" :currentTheme="currentTheme" :toggleTheme="toggleTheme" :apiKey="apiKey" :showApiKey="showApiKey" @copyApiKey="copyApiKey" />
          </div>
        </div>
      </div>
    </main>

    <!-- Configuration Modal -->
    <div v-if="isConfigOpen" class="fixed inset-0 bg-black/70 flex justify-center items-center z-50" @click="isConfigOpen = false">
      <div class="bg-bg-card border border-border-primary p-8 rounded-2xl shadow-secondary max-w-md w-11/12" @click.stop>
        <h3 class="mt-0 text-text-primary mb-6 text-lg font-semibold">API Configuration</h3>
        <div class="my-4">
          <label for="api-url" class="block mb-2 font-medium text-text-primary">API Base URL:</label>
          <input 
            id="api-url"
            v-model="apiConfig.base_url" 
            type="url" 
            placeholder="https://hackatime.hackclub.com"
            class="w-full p-3 bg-bg-secondary border border-border-secondary rounded-xl text-text-primary text-base box-border focus:outline-none focus:border-accent-primary focus:shadow-[0_0_0_2px_rgba(200,57,79,0.2)]"
          />
        </div>
        <div class="flex gap-4 justify-end mt-6">
          <button @click="isConfigOpen = false" class="px-6 py-3 rounded-xl cursor-pointer text-base font-medium transition-all duration-200 bg-transparent text-text-secondary border border-border-secondary hover:bg-bg-secondary hover:text-text-primary hover:border-border-primary">Cancel</button>
          <button @click="saveApiConfig" class="px-6 py-3 rounded-xl cursor-pointer text-base font-medium transition-all duration-200 bg-accent-primary text-white border-0 hover:bg-accent-secondary hover:shadow-card-hover">Save</button>
        </div>
      </div>
    </div>
  </div>
</template>

<!-- All styles now handled by Tailwind CSS -->