<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrent, onOpenUrl } from "@tauri-apps/plugin-deep-link";
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { api } from "./api";
import Home from "./views/Home.vue";
import Projects from "./views/Projects.vue";
import Settings from "./views/Settings.vue";
import Statistics from "./views/Statistics.vue";
import UserProfileCard from "./components/UserProfileCard.vue";
import CustomTitlebar from "./components/CustomTitlebar.vue";
import WakatimeSetupModal from "./components/WakatimeSetupModal.vue";

if (!(window as any).__hackatimeConsoleWrapped) {
  (window as any).__hackatimeConsoleWrapped = true;
  const originalConsole = { ...console } as any;
  ['debug','info','warn','error'].forEach((lvl) => {
    const orig = (originalConsole as any)[lvl] || originalConsole.log;
    (console as any)[lvl] = (...args: any[]) => {
      try { orig.apply(originalConsole, args); } catch (_) {}
    };
  });
  console.info('[CONSOLE] Console wrapper initialized - logs will be captured');
}

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
const oauthUrl = ref<string | null>(null);
const nextPresenceFetchAllowedAt = ref<number>(0);
const lastPresenceFetchAt = ref<number>(0);

const currentPage = ref<'home' | 'projects' | 'statistics' | 'settings'>('home');

const showWakatimeSetupModal = ref(false);
const wakatimeConfigCheck = ref<any>(null);
const hasCheckedConfigThisSession = ref(false);


const weeklyChartData = computed(() => {
  if (!userStats.value?.weekly_stats?.daily_hours) {
    const dayNames = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];
    const today = new Date();
    return dayNames.map((dayName, index) => {
      const date = new Date(today);
      date.setDate(today.getDate() - (6 - index));
      return {
        date: date.toISOString().split('T')[0],
        day_name: dayName,
        hours: 0,
        percentage: 0
      };
    });
  }
  
  const dailyHours = userStats.value.weekly_stats.daily_hours;
  const maxHours = Math.max(...Object.values(dailyHours).map((day: any) => day.hours), 1);
  
  return Object.values(dailyHours)
    .sort((a: any, b: any) => new Date(a.date).getTime() - new Date(b.date).getTime())
    .map((day: any) => ({
      ...day,
      percentage: (day.hours / maxHours) * 100
    }));
});


onMounted(async () => {
  await loadAuthState();
  await loadApiConfig();
  await loadHackatimeInfo();
  
  isDevMode.value = apiConfig.value.base_url.includes('localhost') || 
                    apiConfig.value.base_url.includes('127.0.0.1') ||
                    window.location.hostname === 'localhost' ||
                    window.location.hostname === '127.0.0.1';
  
  try {
    const startUrls = await getCurrent();
    if (startUrls && startUrls.length > 0) {
      console.log("App started with deep link:", startUrls);
      const hasOAuthCallback = startUrls.some(url => 
        url.startsWith('hackatime://auth/callback')
      );
      
      if (hasOAuthCallback) {
        console.log("OAuth callback detected, refreshing auth state...");
        setTimeout(async () => {
          await loadAuthState();
        }, 1000);
      }
    }
  } catch (error) {
    console.error("Failed to get current deep link:", error);
  }
  
  try {
    await onOpenUrl((urls) => {
      console.log("Deep link received in frontend:", urls);
      const hasOAuthCallback = urls.some(url => 
        url.startsWith('hackatime://auth/callback')
      );
      
      if (hasOAuthCallback) {
        console.log("OAuth callback detected, refreshing auth state...");
        setTimeout(async () => {
          await loadAuthState();
        }, 1000);
      }
    });
  } catch (error) {
    console.error("Failed to set up deep link listener:", error);
  }
  
  window.addEventListener('focus', async () => {
    await loadAuthState();
  });
  
  document.addEventListener('visibilitychange', async () => {
    if (!document.hidden) {
      await loadAuthState();
    }
  });
  
  if (authState.value.is_authenticated) {
    await loadPresenceData();
    startPresenceRefresh();
  }
  
  checkForUpdatesAndInstall();
});

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
      
      await loadUserData();
    } else {
      console.log("No saved auth state found, getting current state");
      authState.value = await invoke("get_auth_state");
      console.log("Current auth state:", authState.value);
      
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
    
    try {
      userStats.value = await invoke("get_dashboard_stats", { apiConfig: apiConfig.value });
    } catch (error) {
      console.error("Failed to load user dashboard stats:", error);
    }
    
    await loadApiKey();
    
    await new Promise(resolve => setTimeout(resolve, 500));
    
    await checkWakatimeConfig();
    
    await loadPresenceData();
    startPresenceRefresh();
  } catch (error) {
    console.error("Failed to load user data:", error);
  }
}

async function checkWakatimeConfig(forceShowModal = false) {
  if (!authState.value.is_authenticated || !apiKey.value) {
    return;
  }
  
  const apiUrl = apiConfig.value.base_url ? `${apiConfig.value.base_url}/api/hackatime/v1` : "https://hackatime.hackclub.com/api/hackatime/v1";
  if (!apiUrl || apiUrl.trim() === "") {
    console.warn("API URL is not set, skipping wakatime config check");
    return;
  }
  
  try {
    const check = await invoke("check_wakatime_config", {
      apiKey: apiKey.value,
      apiUrl: apiUrl,
    }) as any;
    
    wakatimeConfigCheck.value = check;
    
    if (forceShowModal || (!hasCheckedConfigThisSession.value && !check.matches)) {
      showWakatimeSetupModal.value = true;
      hasCheckedConfigThisSession.value = true;
    }
  } catch (error) {
    console.error("Failed to check wakatime config:", error);
  }
}

async function handleWakatimeConfigApplied() {
  showWakatimeSetupModal.value = false;
  
  await checkWakatimeConfig(false);
  
  if (wakatimeConfigCheck.value && !wakatimeConfigCheck.value.matches) {
    alert("Configuration was applied but still doesn't match. Please check the error logs.");
  }
}

async function openWakatimeConfigModal() {
  hasCheckedConfigThisSession.value = true; 
  await checkWakatimeConfig(true);
}

async function loadApiKey() {
  try {
    apiKey.value = await invoke("get_api_key", { apiConfig: apiConfig.value });
  } catch (error) {
    console.error("Failed to load API key:", error);
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
  if (presenceFetchInProgress.value) {
    return;
  }
  
  const now = Date.now();
  if (now < nextPresenceFetchAllowedAt.value) {
    return;
  }

  presenceFetchInProgress.value = true;
  try {
    presenceData.value = await invoke("get_latest_heartbeat", { 
      apiConfig: apiConfig.value 
    });
    lastPresenceFetchAt.value = Date.now();
    console.log("Heartbeat data fetched from backend:", presenceData.value);
  } catch (error: any) {
    console.error("Failed to load presence data:", error);
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
  if (presenceRefreshInterval.value) {
    clearInterval(presenceRefreshInterval.value);
    presenceRefreshInterval.value = null;
  }
  presenceRefreshInterval.value = setInterval(loadPresenceData, 60000);
  console.log("Started heartbeat refresh interval (every 60 seconds)");
}

function stopPresenceRefresh() {
  if (presenceRefreshInterval.value) {
    clearInterval(presenceRefreshInterval.value);
    presenceRefreshInterval.value = null;
  }
}


async function authenticate() {
  isLoading.value = true;
  oauthUrl.value = null; 
  try {
    const url = await invoke("authenticate_with_rails", { apiConfig: apiConfig.value });
    oauthUrl.value = url as string;
    console.log("OAuth URL:", url);
  } catch (error) {
    console.error("Authentication failed:", error);
    alert("Authentication failed: " + (error instanceof Error ? error.message : String(error)));
  } finally {
    isLoading.value = false;
  }
}

async function openOAuthUrlManually() {
  if (oauthUrl.value) {
    try {
      const { openUrl } = await import("@tauri-apps/plugin-opener");
      await openUrl(oauthUrl.value);
    } catch (error) {
      console.error("Failed to open URL manually:", error);
      try {
        await navigator.clipboard.writeText(oauthUrl.value);
        alert("Failed to open link. URL copied to clipboard!");
      } catch (clipError) {
        alert(`Failed to open link. Please visit: ${oauthUrl.value}`);
      }
    }
  }
}

async function logout() {
  try {
    stopPresenceRefresh();
    await invoke("logout");
    hasCheckedConfigThisSession.value = false;
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
    
    await invoke("authenticate_with_direct_oauth", { 
      oauthToken: directOAuthToken.value,
      apiConfig: apiConfig.value 
    });
    
    console.log("Direct OAuth auth successful!");
    await loadAuthState();
    
    if (authState.value.is_authenticated) {
      try {
        await invoke("save_auth_state", { authState: authState.value });
        console.log("Auth state saved after direct OAuth authentication");
      } catch (error) {
        console.error("Failed to save auth state after direct OAuth:", error);
      }
    }
    
    directOAuthToken.value = "";
    alert("Authentication successful! You are now logged in.");
  } catch (error) {
    console.error("Direct OAuth auth failed:", error);
    alert("Direct OAuth auth failed: " + error);
  } finally {
    isLoading.value = false;
  }
}

async function checkForUpdatesAndInstall() {
  try {
    console.info('[AUTO-UPDATE] Checking for updates...');
    const update = await check();
    
    if (update) {
      console.info(`[AUTO-UPDATE] Update available: ${update.version}`);
      console.info('[AUTO-UPDATE] Downloading and installing update...');
      
      let downloaded = 0;
      let contentLength = 0;
      
      await update.downloadAndInstall((event) => {
        switch (event.event) {
          case 'Started':
            contentLength = event.data.contentLength ?? 0;
            console.info(`[AUTO-UPDATE] Started downloading ${event.data.contentLength} bytes`);
            break;
          case 'Progress':
            downloaded += event.data.chunkLength;
            const percentage = contentLength > 0 ? Math.round((downloaded / contentLength) * 100) : 0;
            console.info(`[AUTO-UPDATE] Download progress: ${percentage}% (${downloaded} / ${contentLength} bytes)`);
            break;
          case 'Finished':
            console.info('[AUTO-UPDATE] Download finished');
            break;
        }
      });
      
      console.info('[AUTO-UPDATE] Update installed successfully. Restarting app...');
      await relaunch();
    } else {
      console.info('[AUTO-UPDATE] No updates available - app is up to date');
    }
  } catch (error) {
    console.error('[AUTO-UPDATE] Auto-update check failed:', error);
  }
}
</script>

<template>
  <div class="flex flex-col h-screen text-text-primary font-sans outfit app-window" style="background-color: #322433;">
    <CustomTitlebar />
    
    <div class="flex flex-1 overflow-hidden">
      <aside class="w-64 min-w-64 flex flex-col p-0 shadow-xl relative overflow-hidden" style="background-color: #3D2C3E;">
      <div class="absolute left-0 top-[76px] w-full pointer-events-none z-0">
        <div class="absolute left-[63px] top-[616.5px] text-[36px] text-black opacity-20 font-light whitespace-nowrap" style="font-family: 'Outfit', sans-serif;">
          01:55:58
        </div>
        
        <img src="/src/assets/suits-icons.svg" alt="" class="absolute left-[200px] top-0 w-[84px] h-[17.778px]" />
        
        <img src="/src/assets/decorative-lines.svg" alt="" class="absolute left-0 top-[377px] w-[16px] h-[207px]" />
        
        <img src="/src/assets/decorative-lines.svg" alt="" class="absolute left-[284px] top-[377px] w-[16px] h-[207px]" />
        
      </div>
      
      <div class="relative z-10 flex flex-col h-full">
        <div class="p-6" style="background-color: #3D2C3E;">
          <div class="flex justify-center items-center">
            <img src="/src/assets/bird-illustration.svg" alt="Hackatime" class="h-12 w-auto" />
          </div>
        </div>
        
        <nav class="flex-1 py-4 px-6 space-y-5">
          <button 
            @click="currentPage = 'home'" 
            class="pushable w-full"
            :class="currentPage === 'home' ? 'pushable-active' : 'pushable-inactive'"
            style="font-family: 'Outfit', sans-serif;"
          >
            <span 
              class="front w-full h-16 rounded-lg border-2 border-[rgba(0,0,0,0.35)] flex items-center px-4 text-xl font-bold"
              :style="currentPage === 'home' ? 'background: linear-gradient(135deg, #E99682 0%, #EB9182 33%, #E88592 66%, #E883AE 100%); color: white;' : 'background-color: #543c55; color: white;'"
            >
              <svg class="w-8 h-8 mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2.5">
                <path stroke-linecap="round" stroke-linejoin="round" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"></path>
              </svg>
              <span class="ml-auto">home</span>
            </span>
          </button>
          
          <!-- Projects button -->
          <button 
            @click="currentPage = 'projects'" 
            class="pushable w-full"
            :class="currentPage === 'projects' ? 'pushable-active' : 'pushable-inactive'"
            style="font-family: 'Outfit', sans-serif;"
          >
            <span 
              class="front w-full h-16 rounded-lg border-2 border-[rgba(0,0,0,0.35)] flex items-center px-4 text-xl font-bold"
              :style="currentPage === 'projects' ? 'background: linear-gradient(135deg, #E99682 0%, #EB9182 33%, #E88592 66%, #E883AE 100%); color: white;' : 'background-color: #543c55; color: white;'"
            >
              <svg class="w-8 h-8 mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2.5">
                <path stroke-linecap="round" stroke-linejoin="round" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"></path>
              </svg>
              <span class="ml-auto">projects</span>
            </span>
          </button>
          
          <!-- Statistics button (renamed from friends in Figma, keeping your existing page) -->
          <button 
            @click="currentPage = 'statistics'" 
            class="pushable w-full"
            :class="currentPage === 'statistics' ? 'pushable-active' : 'pushable-inactive'"
            style="font-family: 'Outfit', sans-serif;"
          >
            <span 
              class="front w-full h-16 rounded-lg border-2 border-[rgba(0,0,0,0.35)] flex items-center px-4 text-xl font-bold"
              :style="currentPage === 'statistics' ? 'background: linear-gradient(135deg, #E99682 0%, #EB9182 33%, #E88592 66%, #E883AE 100%); color: white;' : 'background-color: #543c55; color: white;'"
            >
              <svg class="w-8 h-8 mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2.5">
                <path stroke-linecap="round" stroke-linejoin="round" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"></path>
              </svg>
              <span class="ml-auto">statistics</span>
            </span>
          </button>
        </nav>
        
        <div class="p-6 mt-auto" style="background-color: #3D2C3E;">
          <UserProfileCard 
            v-if="authState.is_authenticated"
            :authState="authState"
            :userData="userData"
            :presenceData="presenceData"
            :apiConfig="apiConfig"
            @openSettings="currentPage = 'settings'"
          />
        </div>
      </div>
    </aside>

    <!-- Main Content Area -->
    <main class="flex-1 p-6 overflow-y-auto min-w-0">
      <!-- Home Page Layout -->
      <div v-if="currentPage === 'home'" class="flex h-full gap-6 min-h-0 responsive-stack">
        <!-- Main Home Content (Left Side - 2/3) -->
        <div class="flex-1 flex flex-col min-w-0">
          <Home 
            :authState="authState"
            :apiConfig="apiConfig"
            :userData="userData"
            :userStats="userStats"
            :weeklyChartData="weeklyChartData"
            :isLoading="isLoading"
            :isDevMode="isDevMode"
            :oauthUrl="oauthUrl"
            v-model:directOAuthToken="directOAuthToken"
            @authenticate="authenticate"
            @handleDirectOAuthAuth="handleDirectOAuthAuth"
            @openOAuthUrlManually="openOAuthUrlManually"
          />
        </div>

        <!-- Leaderboard Sidebar (Right Side - 1/3) -->
        <div v-if="authState.is_authenticated && userStats" class="w-64 min-w-64 flex flex-col responsive-full-width">
          <div class="card-3d-app h-full">
            <div class="rounded-[8px] border border-black p-4 card-3d-app-front h-full flex flex-col" style="background-color: #3D2C3E;">
            <div class="flex items-center justify-between mb-4">
              <h2 class="text-white text-[16px] font-bold italic m-0" style="font-family: 'Outfit', sans-serif;">
                leaderboard
              </h2>
              <div class="flex gap-2 text-[10px]" style="font-family: 'Outfit', sans-serif;">
                <span class="text-white underline cursor-pointer">friends</span>
                <span class="text-white cursor-pointer">global</span>
              </div>
            </div>
            <!-- Leaderboard content would go here -->
          </div>
          </div>
        </div>
      </div>
      
      <!-- Statistics Page Layout (full page) -->
      <div v-else-if="currentPage === 'statistics'" class="flex flex-col h-full">
        <Statistics :apiConfig="apiConfig" />
      </div>

      <!-- Settings Page Layout (no outer card) -->
      <div v-else-if="currentPage === 'settings'" class="flex flex-col h-full">
        <Settings 
          :apiKey="apiKey" 
          v-model:showApiKey="showApiKey" 
          @copyApiKey="copyApiKey" 
          @logout="logout" 
          @checkWakatimeConfig="openWakatimeConfigModal"
        />
      </div>

      <!-- Projects Page Layout -->
      <div v-else class="flex flex-col h-full">
        <Projects :apiConfig="apiConfig" />
      </div>
    </main>
    </div>

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

    <!-- Wakatime Setup Modal -->
    <WakatimeSetupModal
      v-if="showWakatimeSetupModal && wakatimeConfigCheck && apiKey"
      :api-key="apiKey"
      :api-url="apiConfig.base_url ? `${apiConfig.base_url}/api/hackatime/v1` : 'https://hackatime.hackclub.com/api/hackatime/v1'"
      :config-check="wakatimeConfigCheck"
      @close="showWakatimeSetupModal = false"
      @applied="handleWakatimeConfigApplied"
    />
  </div>
</template>

<style scoped>
.app-window {
  border-radius: 12px;
  overflow: hidden;
  height: 100vh;
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

.pushable-inactive {
  background-color: #2A1F2B;
}

.front {
  display: flex;
  align-items: center;
  border-radius: 12px;
  transform: translateY(-6px);
  transition: transform 0.1s ease;
  position: relative;
}

.pushable:active .front {
  transform: translateY(-2px);
}

/* 3D Card Effect for App-level cards */
.card-3d-app {
  position: relative;
  border-radius: 8px;
  padding: 0;
}

.card-3d-app::before {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: 8px;
  background-color: #2A1F2B;
  z-index: 0;
}

.card-3d-app-front {
  position: relative;
  transform: translateY(-6px);
  z-index: 1;
}
</style>