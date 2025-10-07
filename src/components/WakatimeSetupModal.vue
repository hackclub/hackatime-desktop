<script setup lang="ts">
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';

interface Props {
  apiKey: string;
  apiUrl: string;
  configCheck: any;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  close: [];
  applied: [];
}>();

const isApplying = ref(false);
const showExplanation = ref(false);

interface DiffLine {
  type: 'same' | 'removed' | 'added' | 'header';
  content: string;
  lineNumber?: number;
}

const diffLines = computed(() => {
  const lines: DiffLine[] = [];
  
  if (!props.configCheck) {
    return lines;
  }
  
  const expectedLines = props.configCheck.expected_content.split('\n');
  const actualLines = props.configCheck.actual_content 
    ? props.configCheck.actual_content.split('\n')
    : [];
  
  
  lines.push({
    type: 'header',
    content: props.configCheck.config_path,
  });
  
  if (!props.configCheck.exists) {
    
    lines.push({
      type: 'header',
      content: 'New file',
    });
    expectedLines.forEach((line: string, index: number) => {
      lines.push({
        type: 'added',
        content: line,
        lineNumber: index + 1,
      });
    });
  } else {
    
    const maxLines = Math.max(expectedLines.length, actualLines.length);
    
    for (let i = 0; i < maxLines; i++) {
      const expectedLine = expectedLines[i] || '';
      const actualLine = actualLines[i] || '';
      
      if (expectedLine === actualLine) {
        lines.push({
          type: 'same',
          content: expectedLine,
          lineNumber: i + 1,
        });
      } else {
        if (actualLine) {
          lines.push({
            type: 'removed',
            content: actualLine,
            lineNumber: i + 1,
          });
        }
        if (expectedLine) {
          lines.push({
            type: 'added',
            content: expectedLine,
            lineNumber: i + 1,
          });
        }
      }
    }
  }
  
  return lines;
});

async function applyConfig() {
  isApplying.value = true;
  try {
    
    const apiUrl = props.apiUrl || "https://hackatime.hackclub.com/api/hackatime/v1";
    await invoke('apply_wakatime_config', {
      apiKey: props.apiKey,
      apiUrl: apiUrl,
    });
    emit('applied');
  } catch (error) {
    console.error('Failed to apply config:', error);
    alert('Failed to apply configuration: ' + error);
  } finally {
    isApplying.value = false;
  }
}
</script>

<template>
  <div class="fixed inset-0 bg-black/80 flex justify-center items-start z-50 pt-16 pb-8 px-8" @click="emit('close')">
    <div class="bg-[#3D2C3E] border-2 border-[rgba(0,0,0,0.35)] rounded-2xl shadow-2xl max-w-5xl w-full flex-1 max-h-full flex flex-col" @click.stop>
      <!-- Header -->
      <div class="p-6 border-b border-[rgba(0,0,0,0.2)]">
        <div class="flex items-start justify-between mb-3">
          <div class="flex items-center gap-3">
            <div class="w-12 h-12 rounded-full flex items-center justify-center" style="background: linear-gradient(135deg, #E99682 0%, #EB9182 33%, #E88592 66%, #E883AE 100%);">
              <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"></path>
              </svg>
            </div>
            <div>
              <h2 class="text-3xl font-bold text-white m-0 leading-none" style="font-family: 'Outfit', sans-serif;">
                get started
              </h2>
              <p class="text-white/50 text-sm mt-1" style="font-family: 'Outfit', sans-serif;">
                Step 1 of 1
              </p>
            </div>
          </div>
        </div>
        <p class="text-white/80 text-base" style="font-family: 'Outfit', sans-serif;">
          <template v-if="configCheck?.matches">
            Your system is already configured correctly! You can review the current configuration below.
          </template>
          <template v-else>
            We need to configure your system to connect with Hackatime. This will only take a moment.
          </template>
        </p>
      </div>

      <!-- Content Area -->
      <div class="flex-1 overflow-hidden p-8 flex items-center justify-center min-h-0">
        <!-- Configuration Preview Card -->
        <div class="card-3d w-full max-w-4xl h-full">
          <div class="rounded-[8px] border-2 border-black card-3d-front h-full flex flex-col" style="background-color: #3D2C3E;">
            <div class="p-5 border-b border-[rgba(0,0,0,0.2)] flex-shrink-0 flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="w-10 h-10 rounded-lg flex items-center justify-center" style="background-color: rgba(233, 150, 130, 0.15);">
                  <svg class="w-5 h-5 text-[#E99682]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"></path>
                  </svg>
                </div>
                <div>
                  <h4 class="text-white font-bold text-base m-0" style="font-family: 'Outfit', sans-serif;">
                    <template v-if="configCheck?.matches">
                      Current Configuration
                    </template>
                    <template v-else>
                      {{ configCheck?.exists ? 'Configuration Changes' : 'New Configuration' }}
                    </template>
                  </h4>
                  <p class="text-white/50 text-xs mt-0.5" style="font-family: 'Outfit', sans-serif;">
                    {{ configCheck?.config_path }}
                  </p>
                </div>
              </div>
              <div v-if="configCheck?.exists && !configCheck?.matches" class="flex items-center gap-2 px-3 py-1.5 rounded-lg" style="background-color: rgba(134, 239, 172, 0.1);">
                <svg class="w-3.5 h-3.5 text-[#86efac]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                </svg>
                <span class="text-[#86efac] text-xs font-medium" style="font-family: 'Outfit', sans-serif;">Backup will be created</span>
              </div>
            </div>
            
            <div class="flex-1 overflow-y-auto p-5" style="background-color: rgba(42, 31, 43, 0.3);">
              <div class="rounded-lg overflow-hidden border border-[rgba(0,0,0,0.35)]" style="background-color: #2A1F2B;">
                <div class="diff-viewer font-mono text-xs">
                  <div
                    v-for="(line, index) in diffLines"
                    :key="index"
                    :class="{
                      'diff-header': line.type === 'header',
                      'diff-same': line.type === 'same',
                      'diff-removed': line.type === 'removed',
                      'diff-added': line.type === 'added',
                    }"
                    class="diff-line"
                  >
                    <span v-if="line.type !== 'header'" class="line-number">{{ line.lineNumber || '' }}</span>
                    <span class="line-prefix">
                      <template v-if="line.type === 'removed'">-</template>
                      <template v-else-if="line.type === 'added'">+</template>
                      <template v-else-if="line.type === 'same'"> </template>
                    </span>
                    <span class="line-content">{{ line.content }}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="border-t border-[rgba(0,0,0,0.2)]" style="background-color: rgba(42, 31, 43, 0.3);">
        <!-- Collapsible Explanation -->
        <div class="border-b border-[rgba(0,0,0,0.15)]">
          <button
            @click="showExplanation = !showExplanation"
            class="w-full px-6 py-3 flex items-center justify-between hover:bg-[rgba(255,255,255,0.03)] transition-colors"
          >
            <div class="flex items-center gap-2">
              <svg class="w-4 h-4 text-[#E99682]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
              </svg>
              <span class="text-white/80 text-sm font-medium" style="font-family: 'Outfit', sans-serif;">
                How does this work?
              </span>
            </div>
            <svg 
              class="w-4 h-4 text-white/50 transition-transform duration-200"
              :class="{ 'rotate-180': showExplanation }"
              fill="none" 
              stroke="currentColor" 
              viewBox="0 0 24 24"
            >
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
            </svg>
          </button>
          
          <transition
            enter-active-class="transition-all duration-300 ease-out"
            enter-from-class="opacity-0 max-h-0"
            enter-to-class="opacity-100 max-h-[400px]"
            leave-active-class="transition-all duration-300 ease-in"
            leave-from-class="opacity-100 max-h-[400px]"
            leave-to-class="opacity-0 max-h-0"
          >
            <div v-show="showExplanation" class="overflow-hidden">
              <div class="px-6 pb-4 pt-2">
                <div class="grid grid-cols-3 gap-3 text-xs" style="font-family: 'Outfit', sans-serif;">
                  <div class="p-3 rounded-lg" style="background-color: rgba(233, 150, 130, 0.1);">
                    <div class="flex items-center gap-2 mb-2">
                      <div class="w-5 h-5 rounded-full flex items-center justify-center" style="background-color: rgba(233, 150, 130, 0.2);">
                        <span class="text-[#E99682] font-bold text-xs">1</span>
                      </div>
                      <p class="text-white font-semibold text-xs">Editor Plugins</p>
                    </div>
                    <p class="text-white/60 text-xs leading-relaxed">
                      WakaTime plugins in your editors monitor your coding sessions
                    </p>
                  </div>

                  <div class="p-3 rounded-lg" style="background-color: rgba(232, 133, 146, 0.1);">
                    <div class="flex items-center gap-2 mb-2">
                      <div class="w-5 h-5 rounded-full flex items-center justify-center" style="background-color: rgba(232, 133, 146, 0.2);">
                        <span class="text-[#E88592] font-bold text-xs">2</span>
                      </div>
                      <p class="text-white font-semibold text-xs">Config File</p>
                    </div>
                    <p class="text-white/60 text-xs leading-relaxed">
                      Directs data to Hackatime's server instead of WakaTime's
                    </p>
                  </div>

                  <div class="p-3 rounded-lg" style="background-color: rgba(232, 131, 174, 0.1);">
                    <div class="flex items-center gap-2 mb-2">
                      <div class="w-5 h-5 rounded-full flex items-center justify-center" style="background-color: rgba(232, 131, 174, 0.2);">
                        <span class="text-[#E883AE] font-bold text-xs">3</span>
                      </div>
                      <p class="text-white font-semibold text-xs">Privacy First</p>
                    </div>
                    <p class="text-white/60 text-xs leading-relaxed">
                      Only metadata tracked. Code content is never sent
                    </p>
                  </div>
                </div>
              </div>
            </div>
          </transition>
        </div>

        <!-- Action Area -->
        <div class="p-6">
          <div class="flex justify-end">
            <template v-if="configCheck?.matches">
              <button 
                @click="emit('close')" 
                class="pushable pushable-active"
                style="font-family: 'Outfit', sans-serif;"
              >
                <span class="front px-8 py-3 rounded-lg border-2 border-[rgba(0,0,0,0.35)] font-bold" style="background: linear-gradient(135deg, #E99682 0%, #EB9182 33%, #E88592 66%, #E883AE 100%); color: white;">
                  Close
                </span>
              </button>
            </template>
            <template v-else>
              <button 
                @click="applyConfig" 
                :disabled="isApplying"
                class="pushable pushable-active"
                style="font-family: 'Outfit', sans-serif;"
              >
                <span 
                  class="front px-8 py-3 rounded-lg border-2 border-[rgba(0,0,0,0.35)] font-bold flex items-center gap-2"
                  :style="isApplying ? 'background-color: #666; color: white;' : 'background: linear-gradient(135deg, #E99682 0%, #EB9182 33%, #E88592 66%, #E883AE 100%); color: white;'"
                >
                  <template v-if="!isApplying">
                    <span>Continue</span>
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"></path>
                    </svg>
                  </template>
                  <template v-else>
                    <svg class="animate-spin h-5 w-5" fill="none" viewBox="0 0 24 24">
                      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                    <span>Applying configuration...</span>
                  </template>
                </span>
              </button>
            </template>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 3D Card Effect */
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
  background-color: #2A1F2B;
  z-index: 0;
}

.card-3d-front {
  position: relative;
  transform: translateY(-6px);
  z-index: 1;
}

/* Pushable Buttons */
.pushable {
  border-radius: 12px;
  border: none;
  padding: 0;
  cursor: pointer;
  outline-offset: 4px;
  position: relative;
  background: linear-gradient(135deg, #B85E6D 0%, #B85E6D 33%, #B5546F 66%, #B55389 100%);
}

.pushable:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

.front {
  display: inline-flex;
  align-items: center;
  border-radius: 12px;
  transform: translateY(-4px);
  transition: transform 0.1s ease;
  position: relative;
}

.pushable:active:not(:disabled) .front {
  transform: translateY(-1px);
}

.diff-viewer {
  overflow-x: auto;
}

.diff-line {
  display: flex;
  padding: 0.2rem 0.75rem;
  line-height: 1.6;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', 'Consolas', monospace;
}

.diff-header {
  background-color: rgba(100, 100, 100, 0.2);
  color: #e0e0e0;
  font-weight: bold;
  padding: 0.5rem 0.75rem;
}

.diff-same {
  background-color: transparent;
  color: #d0d0d0;
}

.diff-removed {
  background-color: rgba(220, 38, 38, 0.15);
  color: #fca5a5;
}

.diff-added {
  background-color: rgba(34, 197, 94, 0.15);
  color: #86efac;
}

.line-number {
  min-width: 2.5rem;
  text-align: right;
  color: rgba(255, 255, 255, 0.3);
  margin-right: 0.75rem;
  user-select: none;
  font-size: 0.7rem;
}

.line-prefix {
  min-width: 1.25rem;
  font-weight: bold;
}

.diff-removed .line-prefix {
  color: #fca5a5;
}

.diff-added .line-prefix {
  color: #86efac;
}

.line-content {
  white-space: pre;
  flex: 1;
}
</style>

