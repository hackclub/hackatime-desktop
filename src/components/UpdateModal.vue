<script setup lang="ts">
import { ref, onMounted, computed, nextTick } from 'vue';
import { openUrl } from '@tauri-apps/plugin-opener';

const props = defineProps<{
  version: string;
  currentVersion: string;
}>();

const emit = defineEmits<{
  installNow: [];
  installLater: [];
}>();

const releaseInfo = ref<any>(null);
const loading = ref(true);
const error = ref<string | null>(null);

const releaseUrl = computed(() => 
  `https://github.com/hackclub/hackatime-desktop/releases/tag/app-v${props.version}`
);

const fetchReleaseInfo = async () => {
  loading.value = true;
  error.value = null;
  
  try {
    const response = await fetch(
      `https://api.github.com/repos/hackclub/hackatime-desktop/releases/tags/app-v${props.version}`
    );
    
    if (!response.ok) {
      throw new Error('Failed to fetch release information');
    }
    
    releaseInfo.value = await response.json();
  } catch (err) {
    console.error('Error fetching release info:', err);
    error.value = 'Failed to load release information';
  } finally {
    loading.value = false;
  }
};

const formattedDate = computed(() => {
  if (!releaseInfo.value?.published_at) return '';
  return new Date(releaseInfo.value.published_at).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'long',
    day: 'numeric'
  });
});

const formattedBody = computed(() => {
  if (!releaseInfo.value?.body) return 'No release notes available.';
  
  let text = releaseInfo.value.body;
  
  text = text.replace(/### (.*?)$/gm, '<h3>$1</h3>');
  text = text.replace(/## (.*?)$/gm, '<h2>$1</h2>');
  text = text.replace(/# (.*?)$/gm, '<h2>$1</h2>');
  
  text = text.replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>');
  text = text.replace(/\*([^*]+)\*/g, '<em>$1</em>');
  
  text = text.replace(/`([^`]+)`/g, '<code>$1</code>');
  
  text = text.replace(/\[(.*?)\]\((.*?)\)/g, '<a href="$2" data-external-link>$1</a>');
  
  text = text.replace(/^[\*\-] (.+)$/gm, '<li>$1</li>');
  
  text = text.replace(/(<li>.*<\/li>\n?)+/g, (match: string) => `<ul>${match}</ul>`);
  
  text = text.replace(/\n\n+/g, '</p><p>');
  
  text = `<p>${text}</p>`;
  
  text = text.replace(/<p>\s*<\/p>/g, '');
  text = text.replace(/<p>(<h[23]>)/g, '$1');
  text = text.replace(/(<\/h[23]>)<\/p>/g, '$1');
  text = text.replace(/<p>(<ul>)/g, '$1');
  text = text.replace(/(<\/ul>)<\/p>/g, '$1');
  
  return text;
});

onMounted(async () => {
  await fetchReleaseInfo();
  
  await nextTick();
  
  const links = document.querySelectorAll('[data-external-link]');
  links.forEach(link => {
    link.addEventListener('click', async (e) => {
      e.preventDefault();
      const href = (e.target as HTMLAnchorElement).getAttribute('href');
      if (href) {
        try {
          await openUrl(href);
        } catch (error) {
          console.error('Failed to open link:', error);
        }
      }
    });
  });
});

const handleInstallNow = () => {
  emit('installNow');
};

const handleInstallLater = () => {
  emit('installLater');
};

const openReleaseUrl = async () => {
  try {
    await openUrl(releaseUrl.value);
  } catch (error) {
    console.error('Failed to open release URL:', error);
  }
};
</script>

<template>
  <div 
    class="fixed inset-0 bg-black/80 flex justify-center items-center p-8"
    style="z-index: 10000;"
    @click="handleInstallLater"
  >
    <div class="card-3d max-w-3xl w-full max-h-[90vh]" @click.stop>
      <div class="rounded-[8px] border border-black card-3d-front flex flex-col max-h-[90vh]" style="background-color: #3D2C3E;">
        <div class="p-6 border-b border-[rgba(0,0,0,0.2)] flex-shrink-0">
          <div class="flex items-start justify-between mb-3">
            <div class="flex-1 min-w-0">
              <h2 class="text-3xl font-bold text-white m-0 mb-2" style="font-family: 'Outfit', sans-serif;">
                Update Available
              </h2>
              <div class="flex items-center gap-4 text-white/60 flex-wrap" style="font-family: 'Outfit', sans-serif;">
                <span class="text-base">Version {{ currentVersion }} → {{ version }}</span>
              </div>
            </div>
            <button 
              @click="handleInstallLater"
              class="flex-shrink-0 w-8 h-8 flex items-center justify-center rounded-lg hover:bg-[rgba(255,255,255,0.1)] transition-colors"
            >
              <svg class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
              </svg>
            </button>
          </div>
        </div>

        <div class="flex-1 overflow-y-auto p-6 space-y-6 min-h-0" style="font-family: 'Outfit', sans-serif;">
          <div v-if="loading" class="flex items-center justify-center py-12">
            <div class="animate-spin rounded-full h-8 w-8 border-2 border-[#EB9182] border-t-transparent"></div>
          </div>

          <div v-else-if="error" class="text-center py-12">
            <p class="text-red-400 mb-4">{{ error }}</p>
            <button
              @click="openReleaseUrl"
              class="pushable pushable-active inline-block"
              style="font-family: 'Outfit', sans-serif;"
            >
              <span class="front px-6 py-2 rounded-lg border-2 border-[rgba(0,0,0,0.35)] font-bold" style="background: linear-gradient(135deg, #E99682 0%, #EB9182 33%, #E88592 66%, #E883AE 100%); color: white;">
                View release on GitHub
              </span>
            </button>
          </div>

          <div v-else-if="releaseInfo">
            <div class="bg-[rgba(42,31,43,0.5)] border-2 border-[rgba(0,0,0,0.3)] rounded-lg p-4 mb-6">
              <div class="flex items-center justify-between">
                <div>
                  <div class="text-white/60 text-sm mb-1">Release</div>
                  <div class="text-2xl font-bold text-white">
                    {{ releaseInfo.name || `v${version}` }}
                  </div>
                </div>
                <div class="text-right">
                  <div class="text-white/60 text-sm mb-1">Published</div>
                  <div class="text-white font-medium">{{ formattedDate }}</div>
                </div>
              </div>
            </div>

            <div v-if="releaseInfo.body">
              <h3 class="text-white text-lg font-bold mb-3">Release Notes</h3>
              <div class="release-notes text-white/80 text-sm leading-relaxed">
                <div v-html="formattedBody"></div>
              </div>
            </div>

            <div class="pt-4">
              <button
                @click="openReleaseUrl"
                class="text-[#E99682] hover:text-[#E88592] text-sm flex items-center gap-2 transition-colors"
              >
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"></path>
                </svg>
                <span>View full release notes on GitHub</span>
              </button>
            </div>
          </div>
        </div>

        <div class="p-6 border-t border-[rgba(0,0,0,0.2)] flex gap-3 flex-shrink-0">
          <button
            @click="handleInstallLater"
            class="pushable pushable-inactive flex-1"
            style="font-family: 'Outfit', sans-serif;"
          >
            <span class="front w-full px-6 py-3 rounded-lg border-2 border-[rgba(0,0,0,0.35)] font-bold text-center" style="background-color: #543c55; color: white;">
              Install Later
            </span>
          </button>
          <button
            @click="handleInstallNow"
            class="pushable pushable-active flex-1"
            style="font-family: 'Outfit', sans-serif;"
          >
            <span class="front w-full px-6 py-3 rounded-lg border-2 border-[rgba(0,0,0,0.35)] font-bold text-center" style="background: linear-gradient(135deg, #E99682 0%, #EB9182 33%, #E88592 66%, #E883AE 100%); color: white;">
              Install Now
            </span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

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
  box-shadow: 0 6px 0 #2A1F2B;
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
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 12px;
  transform: translateY(-4px);
  transition: transform 0.1s ease;
  position: relative;
}

.pushable:active .front {
  transform: translateY(-1px);
}

.release-notes :deep(h2) {
  color: white;
  font-size: 1.125rem;
  font-weight: bold;
  margin-top: 1.5rem;
  margin-bottom: 0.75rem;
}

.release-notes :deep(h3) {
  color: white;
  font-size: 1rem;
  font-weight: bold;
  margin-top: 1.25rem;
  margin-bottom: 0.5rem;
}

.release-notes :deep(p) {
  margin-bottom: 0.75rem;
  line-height: 1.6;
}

.release-notes :deep(ul) {
  list-style-type: disc;
  padding-left: 1.5rem;
  margin: 0.75rem 0;
}

.release-notes :deep(li) {
  margin: 0.25rem 0;
  line-height: 1.5;
}

.release-notes :deep(code) {
  background: rgba(0, 0, 0, 0.3);
  padding: 0.125rem 0.375rem;
  border-radius: 0.25rem;
  font-size: 0.875rem;
  font-family: monospace;
  color: #EB9182;
}

.release-notes :deep(strong) {
  color: white;
  font-weight: 600;
}

.release-notes :deep(a) {
  color: #E99682;
  text-decoration: underline;
  cursor: pointer;
}

.release-notes :deep(a:hover) {
  color: #E88592;
}

.overflow-y-auto::-webkit-scrollbar {
  width: 8px;
}

.overflow-y-auto::-webkit-scrollbar-track {
  background: rgba(42, 31, 43, 0.5);
  border-radius: 4px;
}

.overflow-y-auto::-webkit-scrollbar-thumb {
  background: rgba(233, 150, 130, 0.3);
  border-radius: 4px;
}

.overflow-y-auto::-webkit-scrollbar-thumb:hover {
  background: rgba(233, 150, 130, 0.5);
}
</style>

