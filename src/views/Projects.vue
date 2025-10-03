<template>
  <div class="min-h-72">
    <!-- Loading State -->
    <div v-if="isLoading" class="flex items-center justify-center h-64">
      <div class="flex items-center gap-3">
        <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-accent-primary"></div>
        <p class="text-text-secondary">Loading projects...</p>
      </div>
    </div>

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
      <div class="flex justify-between items-center mb-6">
        <h3 class="text-lg font-semibold text-text-primary">Your Projects</h3>
        <div class="text-sm text-text-secondary">
          {{ projects.length }} project{{ projects.length !== 1 ? 's' : '' }}
        </div>
      </div>

      <div class="grid gap-4">
        <div 
          v-for="project in projects" 
          :key="project.name"
          class="bg-bg-secondary border border-border-primary rounded-xl p-4 hover:border-accent-primary transition-colors cursor-pointer"
          @click="selectProject(project)"
        >
          <div class="flex justify-between items-start mb-3">
            <div class="flex-1">
              <h4 class="text-text-primary font-medium text-lg mb-1">{{ project.name }}</h4>
              <div class="flex items-center gap-4 text-sm text-text-secondary">
                <span>{{ project.total_heartbeats }} heartbeats</span>
                <span>{{ formatDuration(project.total_seconds) }}</span>
                <span v-if="project.recent_activity_seconds > 0" class="text-accent-primary">
                  Active recently
                </span>
              </div>
            </div>
            <div class="text-right">
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
              class="px-2 py-1 bg-bg-tertiary text-text-primary text-xs rounded-md"
            >
              {{ language }}
            </span>
            <span 
              v-if="project.languages.length > 3"
              class="px-2 py-1 bg-bg-tertiary text-text-secondary text-xs rounded-md"
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

// Props
const props = defineProps<{
  currentTheme: string;
  toggleTheme: () => void;
  apiConfig: {
    base_url: string;
  };
}>();

// Load projects data
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

// Select a project (for future detailed view)
function selectProject(project: Project) {
  console.log("Selected project:", project.name);
  // TODO: Implement project details view
}

// Format duration helper
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

// Format date helper
function formatDate(dateString: string): string {
  const date = new Date(dateString);
  return date.toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric'
  });
}

// Load projects on mount
onMounted(() => {
  loadProjects();
});
</script>

<!-- All styles now handled by Tailwind CSS -->