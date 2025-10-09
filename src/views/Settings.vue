<template>
  <div class="flex flex-col h-full min-h-0">
    <!-- Header -->
    <div class="mb-6">
      <h1 class="text-[40px] sm:text-[32px] lg:text-[40px] font-bold italic text-white m-0 mb-2" style="font-family: 'Outfit', sans-serif;">
        settings
      </h1>
      <p class="text-[20px] sm:text-[16px] lg:text-[20px] text-white m-0" style="font-family: 'Outfit', sans-serif;">
        tune your experience.
      </p>
    </div>

    <!-- Content Grid -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <!-- Left Column -->
      <div class="flex flex-col gap-6">

        <!-- Preferences -->
        <div class="card-3d">
          <div class="rounded-[8px] border border-black p-5 card-3d-front" style="background-color: #3D2C3E;">
            <h3 class="text-white text-[16px] font-bold m-0 mb-4" style="font-family: 'Outfit', sans-serif;">Preferences</h3>
            <div class="space-y-5">
              <div class="flex items-center justify-between">
                <div>
                  <h4 class="font-medium text-text-primary mb-1">Auto-start</h4>
                  <p class="text-sm text-text-secondary">Start with system</p>
                </div>
                <label class="switch">
                  <input type="checkbox">
                  <span class="slider"></span>
                </label>
              </div>
              <div class="flex items-center justify-between">
                <div>
                  <h4 class="font-medium text-text-primary mb-1">Discord RPC</h4>
                  <p class="text-sm text-text-secondary">Show coding activity in Discord</p>
                </div>
                <label class="switch" :class="{ 'opacity-50 cursor-not-allowed': isLoading }">
                  <input type="checkbox" :checked="discordRpcEnabled" :disabled="isLoading" @change="toggleDiscordRpc">
                  <span class="slider" :class="{ 'animate-pulse': isLoading }"></span>
                </label>
              </div>
              <div class="flex items-center justify-between">
                <div>
                  <h4 class="font-medium text-text-primary mb-1">Notifications</h4>
                  <p class="text-sm text-text-secondary">Show desktop notifications</p>
                </div>
                <label class="switch">
                  <input type="checkbox" checked>
                  <span class="slider"></span>
                </label>
              </div>
            </div>
          </div>
        </div>

        <!-- API Access -->
        <div v-if="apiKey" class="card-3d ph-no-capture">
          <div class="rounded-[8px] border border-black p-5 card-3d-front" style="background-color: #3D2C3E;">
            <h3 class="text-white text-[16px] font-bold m-0 mb-4" style="font-family: 'Outfit', sans-serif;">Your API Key</h3>
            <p class="text-sm text-text-secondary mb-4">Use this key to authenticate with the KubeTime API</p>
            <div class="flex flex-col sm:flex-row gap-3 items-stretch sm:items-center">
              <input 
                :type="showApiKey ? 'text' : 'password'" 
                :value="apiKey" 
                readonly 
                class="flex-1 p-3 bg-[rgba(20,15,21,0.3)] border border-[rgba(50,36,51,0.4)] rounded-xl text-text-primary font-mono text-sm min-w-0 break-all"
              />
              <div class="flex gap-2 flex-shrink-0">
                <button @click="$emit('update:showApiKey', !showApiKey)" class="p-3 border border-[rgba(50,36,51,0.4)] rounded-xl cursor-pointer text-sm min-w-11 transition-all duration-200 bg-[rgba(20,15,21,0.3)] text-text-secondary hover:bg-bg-primary hover:text-text-primary hover:border-accent-primary">
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

      <!-- Right Column -->
      <div class="flex flex-col gap-6">
        <!-- About -->
        <div class="card-3d">
          <div class="rounded-[8px] border border-black p-5 card-3d-front" style="background-color: #3D2C3E;">
            <h3 class="text-white text-[16px] font-bold m-0 mb-4" style="font-family: 'Outfit', sans-serif;">About</h3>
            <div class="space-y-3">
              <div class="flex justify-between">
                <span class="text-text-secondary">Version</span>
                <span 
                  class="text-text-primary font-medium cursor-pointer select-none"
                  title="Tap 5 times for debug"
                  @click="handleVersionClick"
                >
                  {{ appVersion }}
                </span>
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
              <div v-if="updateInfo" class="mt-4 p-4 rounded-lg border border-[rgba(50,36,51,0.4)]" style="background-color: #2A1F2B;">
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

        <!-- WakaTime Configuration -->
        <div class="card-3d">
          <div class="rounded-[8px] border border-black p-5 card-3d-front" style="background-color: #3D2C3E;">
            <div class="flex items-center justify-between">
              <div>
                <h4 class="font-medium text-text-primary mb-1">Setup</h4>
                <p class="text-sm text-text-secondary">Verify and update your .wakatime.cfg file</p>
              </div>
              <button
                @click="$emit('checkWakatimeConfig')"
                class="px-4 py-2 bg-gradient-to-r from-[#E99682] via-[#E88592] to-[#E883AE] text-white rounded-lg hover:opacity-90 transition-opacity font-medium"
              >
                Check Config
              </button>
            </div>
          </div>
        </div>

        <!-- Cache Management -->
        <div class="card-3d">
          <div class="rounded-[8px] border border-black p-5 card-3d-front" style="background-color: #3D2C3E;">
            <div class="flex items-center justify-between">
              <div>
                <h4 class="font-medium text-text-primary mb-1">Statistics Cache</h4>
                <p class="text-sm text-text-secondary">Clear cached statistics data (stored for 30 days)</p>
              </div>
              <button
                @click="clearCache"
                :disabled="isClearingCache"
                class="px-4 py-2 bg-orange-600 text-white rounded-lg hover:bg-orange-700 transition-colors font-medium disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {{ isClearingCache ? 'Clearing...' : 'Clear Cache' }}
              </button>
            </div>
          </div>
        </div>

        <!-- Account -->
        <div class="card-3d">
          <div class="rounded-[8px] border border-black p-5 card-3d-front" style="background-color: #3D2C3E;">
            <div class="flex items-center justify-between">
              <div>
                <h4 class="font-medium text-text-primary mb-1">Sign Out</h4>
                <p class="text-sm text-text-secondary">Log out of your account</p>
              </div>
              <button
                @click="$emit('logout')"
                class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors font-medium"
              >
                Logout
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Debug Modal -->
    <div v-if="showDebugModal" class="modal-backdrop" @click="showDebugModal = false">
      <div class="modal-card-3d" @click.stop>
        <div class="rounded-[8px] border border-black p-6 modal-card-3d-front" style="background-color: #3D2C3E;">
          <div class="flex items-start justify-between mb-4">
            <h3 class="text-white text-[16px] font-bold italic m-0" style="font-family: 'Outfit', sans-serif;">debug console</h3>
            <div class="flex items-center gap-3">
              <button @click="refreshLogs" class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg text-sm font-medium transition-colors duration-200 shadow-md">
                <svg class="w-4 h-4 inline mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"></path>
                </svg>
                refresh
              </button>
              <button @click="clearAllLogs" class="px-4 py-2 bg-orange-600 hover:bg-orange-700 text-white rounded-lg text-sm font-medium transition-colors duration-200 shadow-md">
                <svg class="w-4 h-4 inline mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"></path>
                </svg>
                clear
              </button>
              <button @click="showDebugModal = false" class="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg text-sm font-medium transition-colors duration-200 shadow-md">
                <svg class="w-4 h-4 inline mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                </svg>
                close
              </button>
            </div>
          </div>
          
          <!-- Console Output -->
          <div ref="consoleContainer" class="console-container bg-black rounded-md p-4 font-mono text-[12px] max-h-[60vh] overflow-auto">
            <div v-if="consoleMessages.length === 0" class="text-gray-400">Console ready. Interact with the app or click refresh.</div>
            <div v-for="(message, idx) in consoleMessages" :key="idx" class="console-line flex items-start gap-2 py-0.5">
              <span class="text-gray-500 text-[10px] flex-shrink-0">{{ message.timestamp }}</span>
              <span 
                class="level-badge px-1.5 py-0.5 rounded text-[10px] font-semibold flex-shrink-0"
                :class="getLevelColor(message.level)"
              >
                {{ message.level.toUpperCase() }}
              </span>
              <span 
                class="source-badge px-1.5 py-0.5 rounded text-[10px] font-semibold flex-shrink-0"
                :class="message.source === 'backend' ? 'bg-blue-900 text-blue-200' : 'bg-purple-900 text-purple-200'"
              >
                {{ message.source }}
              </span>
              <span 
                class="message-content flex-1 break-words whitespace-pre-wrap"
                :class="getMessageColor(message.level)"
              >
                {{ message.message }}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getVersion } from '@tauri-apps/api/app';
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
defineProps<{
  apiKey: string | null
  showApiKey: boolean
}>()

const emit = defineEmits<{
  copyApiKey: []
  'update:showApiKey': [value: boolean]
  logout: []
  checkWakatimeConfig: []
}>()

const discordRpcEnabled = ref(false);
const isLoading = ref(false);
const appVersion = ref('...');
const isClearingCache = ref(false);

const updateStatus = ref<'idle' | 'checking' | 'available' | 'latest' | 'error'>('idle');
const updateInfo = ref<{
  version: string;
  notes?: string;
  date?: string;
} | null>(null);
const isInstallingUpdate = ref(false);

const versionTapCount = ref(0);
const showDebugModal = ref(false);
const userIdForDebug = ref<string | null>(null);
const backendLogsError = ref(false);
type LogLevel = 'debug' | 'info' | 'warn' | 'error';
type LogSource = 'frontend' | 'backend';
type LogEntry = { ts: number; level: LogLevel; source: LogSource; message: string };
const frontendLogs = ref<LogEntry[]>([]);
const backendLogs = ref<LogEntry[]>([]);
const platformInfo = ref<any>(null);

const consoleContainer = ref<HTMLElement | null>(null);

const consoleMessages = ref<Array<{
  timestamp: string;
  level: LogLevel;
  source: LogSource;
  message: string;
}>>([]);

function getAsciiArt(): string {
  return [
    "                                   ,'.r?!,'!'",
    "                               ':?,:'   ':'.r:",
    "                             ''',`           .:r'",
    "                            .r''              \`r?",
    "                          ,:}.'      '~~[[.\`   \`:[",
    "                         !r'S.~Y    ,?:7'C:~    '![",
    "                         :yYd?~}    '7.C:,~\`     :r",
    "                          N. ',\`     '!   ~:     ,.}",
    "                          ![\`            .:,      :}",
    "                          \`~!'          .r!       [:'",
    "                    :~.     !,.'.  !'.,r?:         r~",
    "              ',:'', '!.     \`r'\`.\`:\`             YY",
    "              .?!.,:[~~}:        ,[                ,!",
    "           ,   r:    ~kC,        :~                !!:",
    "         .}!  ,['      ~rr\`     \`!'                 }:",
    "        ~[~   :~        \`!}'    !['                 !~",
    "       ~~    [r\`          :?~  \`[r                  '[.",
    "      C'.  ,?~\`            .?, '['                   ?:",
    "       :,7?!'               \`!,[[                    ~7'",
    "    7. \`:},.                 '7?'                     7,",
    "    .?}' ,\`r7:'               ::.                     ~?                  .!",
    "      \`:::   ,r7:'             '!:             !r     !~:               ':Y~",
    "               \`,~[!'           [!           \`[~'      ':C            \`'!r,:",
    "                  '~[rr,                    'r~        !d'!          :!!\`~ '",
    "                     r!YY~                 ,[,         \`7,,'       ':7,  '.'",
    "                     ,~                   .[!           !'!       ![[\`   :~",
    "                     ![                 \`~r'            7[?     \`.?,    .~:",
    "                      C                 [.\`            ::y:    ,!'      ,}",
    "                     !.                 \`,             !7~~Y ':!       :r\`",
    "                     7[                               \`rr Yr,~\`       .r\`",
    "                     r:                               !7\` }!         \`}:",
    "                     :k'                k\`            7[  !Y        \`:r",
    "                      }'              \`\`k}.~~,?r?::, ~r   \`?[      '!~",
    "                      ~[              ':k!.      !~[?7:    7~     .~~",
    "                      ?[                \`':}!,`  \`:!,~     r:   \`.7:",
    "                      '!,                   \`,!:}[~\`       Y   .,~\`",
    "                       ['                                  :[,''",
    "                     \`.,,r:              '!               ,'7.\`",
    "               :\`.\`!!:!':'`              :r~          ?!  !,~",
    "         \`.,,~!...:`                   \`[?           ![ ,!\`",
    "       :,~,'                   '\`     \`r:'          '[r:,'",
    "    'y'.\`!,!\`                 .[,' \`.,'.r            r[:",
    "    !!7  \`'rr             \`~::' '::.\`7,'           :~.",
    "     \`!~   .r?          ,!7\`\`       !!\`           ,:,",
    "      '.\`![` \`[}      ',~[,\`       \`}:\`           '\`~!",
    "         '}:.:}  ''.~~,.         .!!            \`}}",
    "           '':~'}~!~            rr'r!r~!'''    \`![",
    "               \`\`\`.\`            y:.     '.'?[\`':[",
    "                                 ..[!,.    \`~!![",
    "                                    ''.r?!??~.["
  ].join('\n');
}

const originalConsole = { ...console } as any;
function pushFront(level: LogLevel, args: any[]) {
  try {
    const message = args.map(a => typeof a === 'object' ? JSON.stringify(a) : String(a)).join(' ');
    frontendLogs.value.push({ ts: Date.now(), level, source: 'frontend', message });
  } catch (_) {}
}
if (!(window as any).__hackatimeConsoleWrapped) {
  (window as any).__hackatimeConsoleWrapped = true;
  ['debug','info','warn','error'].forEach((lvl) => {
    const key = lvl as LogLevel;
    const orig = (originalConsole as any)[key] || originalConsole.log;
    (console as any)[key] = (...args: any[]) => {
      pushFront(key, args);
      try { orig.apply(originalConsole, args); } catch (_) {}
    };
  });
}


async function scrollToBottom() {
  await nextTick();
  if (consoleContainer.value) {
    consoleContainer.value.scrollTop = consoleContainer.value.scrollHeight;
  }
}

watch([backendLogs, frontendLogs], () => {
  updateConsoleMessages();
  scrollToBottom();
}, { deep: true });

function formatTs(ts: number): string {
  const d = new Date(ts);
  return d.toLocaleTimeString();
}

function updateConsoleMessages() {
  const messages: Array<{
    timestamp: string;
    level: LogLevel;
    source: LogSource;
    message: string;
  }> = [];

  messages.push({
    timestamp: formatTs(Date.now()),
    level: 'info',
    source: 'frontend',
    message: getAsciiArt()
  });

  messages.push({
    timestamp: formatTs(Date.now()),
    level: 'info',
    source: 'frontend',
    message: `Hackatime started on ${platformInfo.value?.platform || 'unknown'} — ${platformInfo.value?.description || 'unknown platform'} · app v${appVersion.value}`
  });

  if (platformInfo.value?.app_data_dir) {
    messages.push({
      timestamp: formatTs(Date.now()),
      level: 'debug',
      source: 'frontend',
      message: `app_data_dir: ${platformInfo.value.app_data_dir}`
    });
  }

  [...backendLogs.value, ...frontendLogs.value]
    .sort((a, b) => a.ts - b.ts)
    .slice(-500) 
    .forEach(entry => {
      messages.push({
        timestamp: formatTs(entry.ts),
        level: entry.level,
        source: entry.source,
        message: entry.message
      });
    });

  consoleMessages.value = messages;
}

function getLevelColor(level: LogLevel): string {
  switch (level) {
    case 'debug': return 'bg-green-900 text-green-200';
    case 'info': return 'bg-blue-900 text-blue-200';
    case 'warn': return 'bg-yellow-900 text-yellow-200';
    case 'error': return 'bg-red-900 text-red-200';
    default: return 'bg-gray-900 text-gray-200';
  }
}

function getMessageColor(level: LogLevel): string {
  switch (level) {
    case 'debug': return 'text-green-300';
    case 'info': return 'text-white';
    case 'warn': return 'text-yellow-300';
    case 'error': return 'text-red-300';
    default: return 'text-gray-300';
  }
}

function clearAllLogs() {
  frontendLogs.value = [];
  backendLogs.value = [];
  consoleMessages.value = [];
  setTimeout(() => {
    updateConsoleMessages();
  }, 10);
}


async function refreshLogs() {
  backendLogsError.value = false;
  try {
    const logs = await invoke('get_recent_logs');
    if (Array.isArray(logs)) {
      backendLogs.value = logs.map((l: any) => ({
        ts: Number(l.ts) || Date.now(),
        level: (l.level || 'info') as LogLevel,
        source: 'backend',
        message: typeof l.message === 'string' ? l.message : JSON.stringify(l.message)
      }));
    } else {
      backendLogsError.value = true;
    }
  } catch (_) {
    backendLogsError.value = true;
  }
}

async function loadPlatformInfo() {
  try {
    platformInfo.value = await invoke('get_platform_info');
  } catch (_) {
    platformInfo.value = null;
  }
}

window.addEventListener('error', (e) => {
  pushFront('error', [e.message || 'window.onerror']);
});
window.addEventListener('unhandledrejection', (e: PromiseRejectionEvent) => {
  try {
    const reason = (e as any)?.reason;
    pushFront('error', [typeof reason === 'string' ? reason : JSON.stringify(reason)]);
  } catch (_) {
    pushFront('error', ['unhandledrejection']);
  }
});

watch(showDebugModal, (open) => {
  if (open) {
    console.info('debug modal opened');
    loadPlatformInfo();
    refreshLogs();
    setTimeout(() => {
      updateConsoleMessages();
    }, 100);
  }
});

function handleVersionClick() {
  versionTapCount.value += 1;
  if (versionTapCount.value >= 5) {
    versionTapCount.value = 0;
    showDebugModal.value = true;
  }
}

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
    
    discordRpcEnabled.value = !discordRpcEnabled.value;
  } finally {
    isLoading.value = false;
  }
}

function copyApiKey() {
  emit('copyApiKey')
}

async function clearCache() {
  if (isClearingCache.value) return;
  
  isClearingCache.value = true;
  console.info('Starting to clear statistics cache...');
  try {
    await invoke('clear_statistics_cache');
    console.info('Statistics cache cleared successfully! Fresh data will be fetched on next request.');
    
    
    if (showDebugModal.value) {
      await refreshLogs();
    }
  } catch (error) {
    console.error('Failed to clear cache:', error);
  } finally {
    isClearingCache.value = false;
  }
}


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


onMounted(async () => {
  loadDiscordRpcState();
  try {
    appVersion.value = await getVersion();
  } catch (error) {
    console.error('Failed to get app version:', error);
    appVersion.value = '1.0.0';
  }
  try {
    const auth = await invoke('get_auth_state');
    
    
    const email = (auth as any)?.user_info?.emails?.[0];
    userIdForDebug.value = email || (auth as any)?.user_info?.id || 'anonymous';
  } catch (e) {
    userIdForDebug.value = 'unknown';
  }
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
  background: #2A1F2B;
  z-index: 0;
}

.card-3d-front {
  position: relative;
  transform: translateY(-6px);
  z-index: 1;
}

/* Debug modal styles */
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-card-3d {
  position: relative;
  border-radius: 8px;
  padding: 0;
  max-width: 720px;
  width: 92vw;
}

.modal-card-3d::before {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: 8px;
  background: #2A1F2B;
  z-index: 0;
}

.modal-card-3d-front {
  position: relative;
  transform: translateY(-6px);
  z-index: 1;
}

/* Custom slider styles */
.slider {
  background-color: #ffffff2b;
  border-radius: 100px;
  padding: 1px;
  margin: 10px;
  cursor: pointer;
  transition: box-shadow 0.2s cubic-bezier(0.4, 0, 0.2, 1) 0s;
  align-items: center;
  position: relative;
  display: block;
  width: 51px;
  height: 29px;
  box-shadow: rgba(0, 0, 0, 0.62) 0px 0px 5px inset, rgba(0, 0, 0, 0.21) 0px 0px 0px 24px inset,
        #22cc3f 0px 0px 0px 0px inset, rgba(224, 224, 224, 0.45) 0px 1px 0px 0px;
}

.slider::after {
  content: "";
  display: flex;
  top: 2.3px;
  left: 2px;
  width: 26px;
  height: 26px;
  background-color: #e3e3e3;
  border-radius: 200px;
  position: absolute;
  box-shadow: transparent 0px 0px 0px 2px, rgba(0, 0, 0, 0.3) 0px 6px 6px;
  transition: left 300ms cubic-bezier(0.4, 0, 0.2, 1) 0s, background-color 300ms cubic-bezier(0.4, 0, 0.2, 1) 0s;
  will-change: left, background-color;
}

.switch input[type="checkbox"]:checked + .slider {
  box-shadow: rgba(0, 0, 0, 0.62) 0px 0px 5px inset, #22cc3f 0px 0px 0px 2px inset, #22cc3f 0px 0px 0px 24px inset,
        rgba(224, 224, 224, 0.45) 0px 1px 0px 0px;
}

.switch input[type="checkbox"]:checked + .slider::after {
  left: 24px;
}

.switch input[type="checkbox"] {
  display: none;
}

/* Console styles */
.console-container {
  background: linear-gradient(135deg, #0a0a0a 0%, #1a1a1a 100%);
  border: 1px solid #333;
  box-shadow: inset 0 1px 3px rgba(0,0,0,0.5);
}

.console-line {
  transition: background-color 0.1s ease;
}

.console-line:hover {
  background-color: rgba(255,255,255,0.05);
}

.level-badge, .source-badge {
  font-size: 9px;
  line-height: 1.2;
  min-width: 50px;
  text-align: center;
}

.message-content {
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  line-height: 1.4;
}
</style>