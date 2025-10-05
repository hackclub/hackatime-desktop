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
          class="px-4 py-2 bg-accent-primary text-white rounded-lg hover:bg-accent-secondary transition-colors"
        >
          Retry
        </button>
      </div>
    </div>

    <!-- Projects List -->
    <div v-else-if="projects && projects.length > 0" class="space-y-4">
      <div class="flex justify-between items-center mb-2">
        <div class="text-sm text-text-secondary">
          {{ projects.length }} project{{ projects.length !== 1 ? 's' : '' }}
        </div>
      </div>

      <div class="grid gap-4">
        <div 
          v-for="project in projects" 
          :key="project.name"
          class="card-3d"
          @click="selectProject(project)"
        >
          <div class="rounded-[8px] border-2 border-black p-4 card-3d-front cursor-pointer hover:bg-[#4a3a4b] transition-colors" style="background-color: #3D2C3E;">
            <div class="flex justify-between items-start mb-3">
              <div class="flex-1 min-w-0">
                <h4 class="text-text-primary font-medium text-lg mb-1 truncate">{{ project.name }}</h4>
                <div class="flex items-center gap-4 text-sm text-text-secondary flex-wrap">
                  <span>{{ project.total_heartbeats }} heartbeats</span>
                  <span>{{ formatDuration(project.total_seconds) }}</span>
                  <span v-if="project.recent_activity_seconds > 0" class="text-accent-primary">
                    Active recently
                  </span>
                </div>
              </div>
              <div class="text-right flex-shrink-0">
                <div class="text-lg font-semibold text-accent-primary">
                  {{ (project.total_seconds / 3600).toFixed(1) }}h
                </div>
              </div>
            </div>

            <!-- Languages and Editors -->
            <div class="flex flex-wrap gap-2 mb-3">
              <span 
                v-for="language in project.languages.slice(0, 3)" 
                :key="language"
                class="px-2 py-1 bg-[rgba(50,36,51,0.15)] text-text-primary text-xs rounded-md"
              >
                {{ language }}
              </span>
              <span 
                v-if="project.languages.length > 3"
                class="px-2 py-1 bg-[rgba(50,36,51,0.15)] text-text-secondary text-xs rounded-md"
              >
                +{{ project.languages.length - 3 }} more
              </span>
            </div>

            <!-- Time Range -->
            <div class="text-xs text-text-secondary">
              <span v-if="project.first_heartbeat">
                First: {{ formatDate(project.first_heartbeat) }}
              </span>
              <span v-if="project.last_heartbeat" class="ml-4">
                Last: {{ formatDate(project.last_heartbeat) }}
              </span>
            </div>

            <!-- Repo Link -->
            <div v-if="project.repo_url" class="mt-2">
              <a 
                :href="project.repo_url" 
                target="_blank" 
                rel="noopener noreferrer"
                class="text-accent-primary text-sm hover:underline"
                @click.stop
              >
                View Repository →
              </a>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Empty State -->
    <div v-else class="flex items-center justify-center h-64">
      <div class="text-center">
        <p class="text-text-secondary mb-4">No projects found</p>
        <p class="text-sm text-text-secondary">
          Start coding to see your projects appear here!
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
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

const projects = ref<Project[]>([]);
const isLoading = ref(false);
const error = ref<string | null>(null);


const props = defineProps<{
  apiConfig: {
    base_url: string;
  };
}>();


async function loadProjects() {
  isLoading.value = true;
  error.value = null;
  
  try {
    const response = await invoke("get_projects", { 
      apiConfig: props.apiConfig 
    }) as ProjectsResponse;
    projects.value = response.projects || [];
  } catch (err) {
    console.error("Failed to load projects:", err);
    error.value = err instanceof Error ? err.message : "Failed to load projects";
  } finally {
    isLoading.value = false;
  }
}


function selectProject(project: Project) {
  console.log("Selected project:", project.name);
  
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
    day: 'numeric'
  });
}


onMounted(() => {
  loadProjects();
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
</style>