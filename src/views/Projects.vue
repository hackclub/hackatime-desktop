<template>
  <div class="flex flex-col h-full min-h-0">
    <!-- Header -->
    <div class="mb-6">
      <h1 class="text-[40px] sm:text-[32px] lg:text-[40px] font-bold italic text-white m-0 mb-2" style="font-family: 'Outfit', sans-serif;">
        projects
      </h1>
      <p class="text-[20px] sm:text-[16px] lg:text-[20px] text-white m-0" style="font-family: 'Outfit', sans-serif;">
        explore what you've been building.
      </p>
    </div>

    <!-- Loading State -->
    <RandomLoader v-if="isLoading" />

    <!-- Error State -->
    <div v-else-if="error" class="flex items-center justify-center h-64">
      <div class="text-center">
        <p class="text-accent-danger mb-4">{{ error }}</p>
        <button 
          @click="loadProjects" 
          class="pushable pushable-active"
          style="font-family: 'Outfit', sans-serif;"
        >
          <span class="front px-6 py-2 rounded-lg border-2 border-[rgba(0,0,0,0.35)] font-bold" style="background: linear-gradient(135deg, #E99682 0%, #EB9182 33%, #E88592 66%, #E883AE 100%); color: white;">
            Retry
          </span>
        </button>
      </div>
    </div>

    <!-- Projects List -->
    <div v-else-if="allProjects && allProjects.length > 0" class="flex flex-col h-full min-h-0">
      <!-- Search and Filter Controls -->
      <div class="mb-6 space-y-4 flex-shrink-0">
        <!-- Search Bar -->
        <div class="relative">
          <input 
            v-model="searchQuery"
            type="text" 
            placeholder="Search projects..."
            class="w-full p-3 pl-10 bg-[#3D2C3E] border border-black rounded-lg text-white text-base box-border focus:outline-none focus:border-[#E99682] transition-colors"
            style="font-family: 'Outfit', sans-serif;"
          />
          <svg class="w-5 h-5 absolute left-3 top-1/2 transform -translate-y-1/2 text-white/50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path>
          </svg>
        </div>

        <!-- Filters Bar -->
        <div class="flex flex-wrap gap-3 items-center">
          <!-- Sort Dropdown -->
          <div class="relative" ref="sortDropdownRef">
            <button
              @click.stop="sortDropdownOpen = !sortDropdownOpen"
              class="px-4 py-2 bg-[rgba(61,44,62,0.5)] border border-[rgba(255,255,255,0.1)] rounded-lg text-white text-sm cursor-pointer hover:bg-[rgba(61,44,62,0.8)] transition-colors flex items-center gap-2"
              style="font-family: 'Outfit', sans-serif;"
            >
              <span class="text-white/60">Sort:</span>
              <span>{{ sortByLabel }}</span>
              <svg class="w-4 h-4 transition-transform" :class="{ 'rotate-180': sortDropdownOpen }" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
              </svg>
            </button>
            <div
              v-if="sortDropdownOpen"
              class="absolute top-full left-0 mt-2 w-48 bg-[#3D2C3E] border-2 border-black rounded-lg shadow-lg overflow-hidden z-50 max-h-60 overflow-y-auto"
            >
              <button
                v-for="option in sortOptions"
                :key="option.value"
                @click.stop="selectSort(option.value)"
                class="w-full px-4 py-2 text-left text-white text-sm hover:bg-[rgba(233,150,130,0.2)] transition-colors"
                :class="{ 'bg-[rgba(233,150,130,0.1)] text-[#E99682] font-medium': sortBy === option.value }"
                style="font-family: 'Outfit', sans-serif;"
              >
                {{ option.label }}
              </button>
            </div>
          </div>

          <!-- Language Filter -->
          <div class="relative" ref="languageDropdownRef" v-if="allLanguages.length > 0">
            <button
              @click.stop="languageDropdownOpen = !languageDropdownOpen"
              class="px-4 py-2 bg-[rgba(61,44,62,0.5)] border border-[rgba(255,255,255,0.1)] rounded-lg text-white text-sm cursor-pointer hover:bg-[rgba(61,44,62,0.8)] transition-colors flex items-center gap-2"
              style="font-family: 'Outfit', sans-serif;"
            >
              <span class="text-white/60">Language:</span>
              <span>{{ filterLanguage || 'All' }}</span>
              <svg class="w-4 h-4 transition-transform" :class="{ 'rotate-180': languageDropdownOpen }" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
              </svg>
            </button>
            <div
              v-if="languageDropdownOpen"
              class="absolute top-full left-0 mt-2 w-48 bg-[#3D2C3E] border-2 border-black rounded-lg shadow-lg overflow-hidden z-50 max-h-60 overflow-y-auto custom-scrollbar"
            >
              <button
                @click.stop="selectLanguage('')"
                class="w-full px-4 py-2 text-left text-white text-sm hover:bg-[rgba(233,150,130,0.2)] transition-colors"
                :class="{ 'bg-[rgba(233,150,130,0.1)] text-[#E99682] font-medium': filterLanguage === '' }"
                style="font-family: 'Outfit', sans-serif;"
              >
                All Languages
              </button>
              <button
                v-for="lang in allLanguages"
                :key="lang"
                @click.stop="selectLanguage(lang)"
                class="w-full px-4 py-2 text-left text-white text-sm hover:bg-[rgba(233,150,130,0.2)] transition-colors"
                :class="{ 'bg-[rgba(233,150,130,0.1)] text-[#E99682] font-medium': filterLanguage === lang }"
                style="font-family: 'Outfit', sans-serif;"
              >
                {{ lang }}
              </button>
            </div>
          </div>

          <!-- Results Count -->
          <div class="ml-auto text-sm text-white/60" style="font-family: 'Outfit', sans-serif;">
            {{ filteredProjects.length }} of {{ allProjects.length }} project{{ allProjects.length !== 1 ? 's' : '' }}
          </div>
        </div>
      </div>

      <!-- Projects Grid with Virtual Scrolling -->
      <div class="flex-1 overflow-y-auto min-h-0" ref="scrollContainer">
        <div class="grid gap-4 pt-2 pb-4">
          <div 
            v-for="(project, index) in paginatedProjects" 
            :key="`${project?.name || 'unnamed'}-${index}`"
            class="card-3d"
            @click="selectProject(project)"
          >
            <div class="rounded-[8px] border border-black p-4 card-3d-front cursor-pointer hover:bg-[#4a3a4b] transition-colors" style="background-color: #3D2C3E;">
              <div class="flex justify-between items-start mb-3">
                <div class="flex-1 min-w-0">
                  <h4 class="text-white font-semibold text-lg mb-1 truncate" style="font-family: 'Outfit', sans-serif;">{{ project?.name || 'Unnamed' }}</h4>
                  <div class="flex items-center gap-4 text-sm text-white/60 flex-wrap" style="font-family: 'Outfit', sans-serif;">
                    <span>{{ ((project?.total_heartbeats ?? 0)).toLocaleString() }} heartbeats</span>
                    <span>{{ formatDuration(project?.total_seconds ?? 0) }}</span>
                    <span v-if="project?.recent_activity_seconds && project.recent_activity_seconds > 0" class="text-[#E99682] font-medium">
                      Active recently
                    </span>
                  </div>
                </div>
                <div class="text-right flex-shrink-0">
                  <div class="text-xl font-bold text-[#E99682]" style="font-family: 'Outfit', sans-serif;">
                    {{ ((project?.total_seconds ?? 0) / 3600).toFixed(1) }}h
                  </div>
                </div>
              </div>

              <!-- Languages and Editors -->
              <div class="flex flex-wrap gap-2 mb-3">
                <span 
                  v-for="(language, langIndex) in (project?.languages || []).slice(0, 3)" 
                  :key="`${project?.name}-lang-${langIndex}-${language}`"
                  class="px-2 py-1 bg-[rgba(233,150,130,0.15)] text-[#E99682] text-xs rounded-md font-medium"
                  style="font-family: 'Outfit', sans-serif;"
                >
                  {{ language }}
                </span>
                <span 
                  v-if="(project?.languages || []).length > 3"
                  class="px-2 py-1 bg-[rgba(50,36,51,0.15)] text-white/60 text-xs rounded-md"
                  style="font-family: 'Outfit', sans-serif;"
                >
                  +{{ (project?.languages || []).length - 3 }} more
                </span>
              </div>

              <!-- Time Range -->
              <div class="text-xs text-white/50" style="font-family: 'Outfit', sans-serif;">
                <span v-if="project?.first_heartbeat">
                  First: {{ formatDate(project.first_heartbeat) }}
                </span>
                <span v-if="project?.last_heartbeat" class="ml-4">
                  Last: {{ formatDate(project.last_heartbeat) }}
                </span>
              </div>
            </div>
          </div>
        </div>

        <!-- Load More Button -->
        <div v-if="hasMoreProjects" class="flex justify-center py-4">
          <button 
            @click="loadMore"
            class="pushable pushable-active"
            style="font-family: 'Outfit', sans-serif;"
          >
            <span class="front px-6 py-2 rounded-lg border-2 border-[rgba(0,0,0,0.35)] font-bold" style="background: linear-gradient(135deg, #E99682 0%, #EB9182 33%, #E88592 66%, #E883AE 100%); color: white;">
              Load More
            </span>
          </button>
        </div>
      </div>
    </div>

    <!-- Empty State -->
    <div v-else class="flex items-center justify-center h-64">
      <div class="text-center">
        <svg class="w-16 h-16 mx-auto mb-4 text-white/30" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"></path>
        </svg>
        <p class="text-white/60 mb-2 text-lg" style="font-family: 'Outfit', sans-serif;">No projects found</p>
        <p class="text-sm text-white/40" style="font-family: 'Outfit', sans-serif;">
          Start coding to see your projects appear here!
        </p>
      </div>
    </div>

    <!-- Project Detail Modal -->
    <div 
      v-if="selectedProject" 
      class="fixed inset-0 bg-black/80 flex justify-center items-center z-50 p-8" 
      @click="closeModal"
    >
      <div class="card-3d max-w-3xl w-full max-h-[90vh]" @click.stop>
        <div class="rounded-[8px] border border-black card-3d-front flex flex-col max-h-[90vh]" style="background-color: #3D2C3E;">
          <!-- Modal Header -->
          <div class="p-6 border-b border-[rgba(0,0,0,0.2)] flex-shrink-0">
            <div class="flex items-start justify-between mb-3">
              <div class="flex-1 min-w-0">
                <h2 class="text-3xl font-bold text-white m-0 mb-2 truncate" style="font-family: 'Outfit', sans-serif;">
                  {{ selectedProject?.name || 'Unnamed' }}
                </h2>
                <div class="flex items-center gap-4 text-white/60 flex-wrap" style="font-family: 'Outfit', sans-serif;">
                  <span class="text-base">{{ ((selectedProject?.total_heartbeats ?? 0)).toLocaleString() }} heartbeats</span>
                  <span v-if="selectedProject?.recent_activity_seconds && selectedProject.recent_activity_seconds > 0" class="px-2 py-1 bg-[rgba(233,150,130,0.2)] text-[#E99682] text-sm rounded-md font-medium">
                    Active recently
                  </span>
                </div>
              </div>
              <button 
                @click="closeModal"
                class="flex-shrink-0 w-8 h-8 flex items-center justify-center rounded-lg hover:bg-[rgba(255,255,255,0.1)] transition-colors"
              >
                <svg class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                </svg>
              </button>
            </div>
          </div>

          <!-- Modal Content -->
          <div class="flex-1 overflow-y-auto p-6 space-y-6 min-h-0">
            <!-- Stats Grid -->
            <div class="grid grid-cols-2 gap-4">
              <div class="bg-[rgba(42,31,43,0.5)] border-2 border-[rgba(0,0,0,0.3)] rounded-lg p-4">
                <div class="text-white/60 text-sm mb-1" style="font-family: 'Outfit', sans-serif;">Total Time</div>
                <div class="text-3xl font-bold text-white" style="font-family: 'Outfit', sans-serif;">
                  {{ ((selectedProject?.total_seconds ?? 0) / 3600).toFixed(1) }}h
                </div>
                <div class="text-white/40 text-xs mt-1" style="font-family: 'Outfit', sans-serif;">
                  {{ formatDuration(selectedProject?.total_seconds ?? 0) }}
                </div>
              </div>

              <div class="bg-[rgba(42,31,43,0.5)] border-2 border-[rgba(0,0,0,0.3)] rounded-lg p-4">
                <div class="text-white/60 text-sm mb-1" style="font-family: 'Outfit', sans-serif;">Heartbeats</div>
                <div class="text-3xl font-bold text-white" style="font-family: 'Outfit', sans-serif;">
                  {{ ((selectedProject?.total_heartbeats ?? 0)).toLocaleString() }}
                </div>
                <div class="text-white/40 text-xs mt-1" style="font-family: 'Outfit', sans-serif;">
                  Activity events
                </div>
              </div>
            </div>

            <!-- Languages Section -->
            <div v-if="selectedProject?.languages && selectedProject.languages.length > 0">
              <h3 class="text-white text-lg font-bold mb-3" style="font-family: 'Outfit', sans-serif;">Languages</h3>
              <div class="flex flex-wrap gap-2">
                <span 
                  v-for="(language, langIndex) in selectedProject.languages" 
                  :key="`modal-lang-${langIndex}-${language}`"
                  class="px-3 py-2 bg-[rgba(233,150,130,0.15)] text-[#E99682] text-sm rounded-lg font-medium border-2 border-[rgba(233,150,130,0.3)]"
                  style="font-family: 'Outfit', sans-serif;"
                >
                  {{ language }}
                </span>
              </div>
            </div>

            <!-- Editors Section -->
            <div v-if="selectedProject?.editors && selectedProject.editors.length > 0">
              <h3 class="text-white text-lg font-bold mb-3" style="font-family: 'Outfit', sans-serif;">Editors</h3>
              <div class="flex flex-wrap gap-2">
                <span 
                  v-for="(editor, editorIndex) in selectedProject.editors" 
                  :key="`modal-editor-${editorIndex}-${editor}`"
                  class="px-3 py-2 bg-[rgba(232,133,146,0.15)] text-[#E88592] text-sm rounded-lg font-medium border-2 border-[rgba(232,133,146,0.3)]"
                  style="font-family: 'Outfit', sans-serif;"
                >
                  {{ editor }}
                </span>
              </div>
            </div>

            <!-- Repository Link -->
            <div v-if="selectedProject?.repo_url">
              <a 
                :href="selectedProject.repo_url" 
                target="_blank" 
                rel="noopener noreferrer"
                class="pushable pushable-active w-full block"
                style="font-family: 'Outfit', sans-serif;"
              >
                <span class="front w-full py-3 px-4 rounded-lg border-2 border-[rgba(0,0,0,0.35)] font-bold flex items-center justify-center gap-2" style="background: linear-gradient(135deg, #E99682 0%, #EB9182 33%, #E88592 66%, #E883AE 100%); color: white;">
                  <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"></path>
                  </svg>
                  View Repository
                </span>
              </a>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import RandomLoader from "../components/RandomLoader.vue";

interface Project {
  name: string;
  total_seconds: number;
  total_heartbeats: number;
  languages: string[];
  editors: string[];
  first_heartbeat: string | null;
  last_heartbeat: string | null;
  repo_url: string | null;
  recent_activity_seconds: number;
  recent_activity_formatted: string;
}

interface ProjectsResponse {
  projects: Project[];
  total_count: number;
  time_range: {
    since: string;
    until: string;
  };
}

const allProjects = ref<Project[]>([]);
const isLoading = ref(false);
const error = ref<string | null>(null);
const selectedProject = ref<Project | null>(null);

const searchQuery = ref("");
const sortBy = ref("recent");
const filterLanguage = ref("");

const sortDropdownOpen = ref(false);
const languageDropdownOpen = ref(false);
const sortDropdownRef = ref<HTMLElement | null>(null);
const languageDropdownRef = ref<HTMLElement | null>(null);

const sortOptions = [
  { value: 'recent', label: 'Most Recent' },
  { value: 'time', label: 'Most Time' },
  { value: 'name', label: 'Name (A-Z)' },
  { value: 'heartbeats', label: 'Most Active' }
];

const itemsPerPage = ref(20);
const currentPage = ref(1);
const scrollContainer = ref<HTMLElement | null>(null);

const props = defineProps<{
  apiConfig: {
    base_url: string;
  };
}>();

const sortByLabel = computed(() => {
  const option = sortOptions.find(opt => opt.value === sortBy.value);
  return option ? option.label : 'Sort';
});

const allLanguages = computed(() => {
  const languages = new Set<string>();
  allProjects.value.forEach(project => {
    if (project && project.languages && Array.isArray(project.languages)) {
      project.languages.forEach(lang => {
        if (lang && typeof lang === 'string') {
          languages.add(lang);
        }
      });
    }
  });
  return Array.from(languages).sort();
});

const filteredProjects = computed(() => {
  let filtered = [...allProjects.value].filter(project => project && typeof project === 'object');

  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase();
    filtered = filtered.filter(project => {
      if (!project) return false;
      
      const nameMatch = project.name?.toLowerCase().includes(query);
      const languageMatch = Array.isArray(project.languages) && 
        project.languages.some(lang => lang && typeof lang === 'string' && lang.toLowerCase().includes(query));
      const editorMatch = Array.isArray(project.editors) && 
        project.editors.some(editor => editor && typeof editor === 'string' && editor.toLowerCase().includes(query));
      
      return nameMatch || languageMatch || editorMatch;
    });
  }

  if (filterLanguage.value) {
    filtered = filtered.filter(project => 
      project && Array.isArray(project.languages) && project.languages.includes(filterLanguage.value)
    );
  }

  switch (sortBy.value) {
    case "recent":
      filtered.sort((a, b) => {
        const dateA = a?.last_heartbeat ? new Date(a.last_heartbeat).getTime() : 0;
        const dateB = b?.last_heartbeat ? new Date(b.last_heartbeat).getTime() : 0;
        return dateB - dateA;
      });
      break;
    case "time":
      filtered.sort((a, b) => (b?.total_seconds || 0) - (a?.total_seconds || 0));
      break;
    case "name":
      filtered.sort((a, b) => (a?.name || '').localeCompare(b?.name || ''));
      break;
    case "heartbeats":
      filtered.sort((a, b) => (b?.total_heartbeats || 0) - (a?.total_heartbeats || 0));
      break;
  }

  return filtered;
});

const paginatedProjects = computed(() => {
  const end = currentPage.value * itemsPerPage.value;
  return filteredProjects.value.slice(0, end);
});

const hasMoreProjects = computed(() => {
  return paginatedProjects.value.length < filteredProjects.value.length;
});

function normalizeProject(project: any): Project {
  return {
    name: project?.name || 'Unnamed Project',
    total_seconds: Number(project?.total_seconds) || 0,
    total_heartbeats: Number(project?.total_heartbeats) || 0,
    languages: Array.isArray(project?.languages) ? project.languages : [],
    editors: Array.isArray(project?.editors) ? project.editors : [],
    first_heartbeat: project?.first_heartbeat || null,
    last_heartbeat: project?.last_heartbeat || null,
    repo_url: project?.repo_url || null,
    recent_activity_seconds: Number(project?.recent_activity_seconds) || 0,
    recent_activity_formatted: project?.recent_activity_formatted || ''
  };
}

async function loadProjects() {
  isLoading.value = true;
  error.value = null;
  
  const timeout = setTimeout(() => {
    if (isLoading.value) {
      console.error("Projects loading timed out after 30 seconds");
      isLoading.value = false;
      error.value = "Loading timed out. Please try again.";
    }
  }, 30000);
  
  try {
    console.log("Loading projects with config:", props.apiConfig);
    const response = await invoke("get_projects", { 
      apiConfig: props.apiConfig 
    }) as ProjectsResponse;
    console.log("Projects loaded:", response);
    
    const projects = response?.projects || [];
    allProjects.value = projects.map(normalizeProject).filter(p => p.name && p.name !== 'Unnamed Project');
  } catch (err) {
    console.error("Failed to load projects:", err);
    error.value = err instanceof Error ? err.message : String(err);
  } finally {
    clearTimeout(timeout);
    isLoading.value = false;
  }
}

function selectProject(project: Project) {
  selectedProject.value = project;
}

function closeModal() {
  selectedProject.value = null;
}

function selectSort(value: string) {
  sortBy.value = value;
  sortDropdownOpen.value = false;
}

function selectLanguage(lang: string) {
  filterLanguage.value = lang;
  languageDropdownOpen.value = false;
}

function loadMore() {
  currentPage.value++;
}

function handleClickOutside(event: MouseEvent) {
  const target = event.target as Node;
  
  if (sortDropdownRef.value && !sortDropdownRef.value.contains(target)) {
    sortDropdownOpen.value = false;
  }
  
  if (languageDropdownRef.value && !languageDropdownRef.value.contains(target)) {
    languageDropdownOpen.value = false;
  }
}

function formatDuration(seconds: number): string {
  if (!seconds || seconds <= 0) return "0m";
  
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  
  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  } else {
    return `${minutes}m`;
  }
}

function formatDate(dateString: string): string {
  const date = new Date(dateString);
  return date.toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  });
}

onMounted(() => {
  loadProjects();
  document.addEventListener('click', handleClickOutside);
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
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

.overflow-y-auto::-webkit-scrollbar,
.custom-scrollbar::-webkit-scrollbar {
  width: 8px;
}

.overflow-y-auto::-webkit-scrollbar-track,
.custom-scrollbar::-webkit-scrollbar-track {
  background: rgba(42, 31, 43, 0.5);
  border-radius: 4px;
}

.overflow-y-auto::-webkit-scrollbar-thumb,
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(233, 150, 130, 0.3);
  border-radius: 4px;
}

.overflow-y-auto::-webkit-scrollbar-thumb:hover,
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(233, 150, 130, 0.5);
}
</style>