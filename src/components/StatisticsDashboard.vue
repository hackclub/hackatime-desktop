<template>
  <div v-if="statisticsData" class="space-y-6">
    <!-- Trends Section -->
    <div>
      <h2 class="text-xl font-semibold text-text-primary mb-4">Trends</h2>
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <StatisticsCard
          v-for="trend in statisticsData.trends"
          :key="trend.title"
          :title="trend.title"
          :value="trend.value"
          :change="trend.change"
          :change-type="trend.change_type"
          :period="trend.period"
          :icon="trend.icon"
          :color="trend.color"
        />
      </div>
    </div>

    <!-- Charts Section -->
    <div>
      <h2 class="text-xl font-semibold text-text-primary mb-4">Analytics</h2>
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <ChartComponent
          v-for="chart in statisticsData.charts"
          :key="chart.id"
          :title="chart.title"
          :chart-type="chart.chart_type"
          :data="chart.data"
          :period="chart.period"
          :color-scheme="chart.color_scheme"
        />
      </div>
    </div>

    <!-- Programmer Class Section -->
    <div>
      <h2 class="text-xl font-semibold text-text-primary mb-4">Your Programmer Class</h2>
      <div class="bg-bg-card border border-border-primary rounded-2xl p-6">
        <div class="flex items-center justify-between mb-4">
          <div>
            <h3 class="text-2xl font-bold text-text-primary">{{ statisticsData.programmer_class.class_name }}</h3>
            <p class="text-text-secondary">{{ statisticsData.programmer_class.description }}</p>
          </div>
          <div class="text-right">
            <div class="text-sm font-semibold px-3 py-1 rounded-full" :style="{ backgroundColor: statisticsData.programmer_class.color + '20', color: statisticsData.programmer_class.color }">
              {{ statisticsData.programmer_class.level }}
            </div>
          </div>
        </div>
        <div class="flex flex-wrap gap-2">
          <span 
            v-for="tech in statisticsData.programmer_class.technologies" 
            :key="tech"
            class="px-3 py-1 bg-bg-secondary text-text-primary rounded-lg text-sm font-medium"
          >
            {{ tech }}
          </span>
        </div>
      </div>
    </div>

    <!-- Insights Section -->
    <div>
      <h2 class="text-xl font-semibold text-text-primary mb-4">Insights</h2>
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <InsightCard
          v-for="insight in statisticsData.insights"
          :key="insight.title"
          :title="insight.title"
          :description="insight.description"
          :value="insight.value"
          :trend="insight.trend"
          :icon="insight.icon"
          :color="insight.color"
        />
      </div>
    </div>
  </div>

  <!-- Loading State -->
  <div v-else-if="isLoading" class="space-y-6">
    <div class="animate-pulse">
      <div class="h-6 bg-bg-secondary rounded w-32 mb-4"></div>
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div v-for="i in 3" :key="i" class="bg-bg-card border border-border-primary rounded-2xl p-6">
          <div class="h-4 bg-bg-secondary rounded w-24 mb-4"></div>
          <div class="h-8 bg-bg-secondary rounded w-16 mb-2"></div>
          <div class="h-3 bg-bg-secondary rounded w-20"></div>
        </div>
      </div>
    </div>
  </div>

  <!-- Error State -->
  <div v-else-if="error" class="bg-red-50 border border-red-200 rounded-2xl p-6">
    <div class="flex items-center">
      <div class="text-red-500 text-xl mr-3">⚠️</div>
      <div>
        <h3 class="text-red-800 font-semibold">Failed to load statistics</h3>
        <p class="text-red-600 text-sm">{{ error }}</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import StatisticsCard from './StatisticsCard.vue';
import ChartComponent from './ChartComponent.vue';
import InsightCard from './InsightCard.vue';

interface StatisticsData {
  trends: Array<{
    title: string;
    value: string;
    change: string;
    change_type: string;
    period: string;
    icon: string;
    color: string;
  }>;
  charts: Array<{
    id: string;
    title: string;
    chart_type: string;
    data: any;
    period: string;
    color_scheme: string;
  }>;
  insights: Array<{
    title: string;
    description: string;
    value: string;
    trend: string;
    icon: string;
    color: string;
  }>;
  programmer_class: {
    class_name: string;
    description: string;
    technologies: string[];
    level: string;
    color: string;
  };
}

interface Props {
  apiConfig: {
    base_url: string;
  };
}

const props = defineProps<Props>();

const statisticsData = ref<StatisticsData | null>(null);
const isLoading = ref(false);
const error = ref<string | null>(null);

const loadStatistics = async () => {
  isLoading.value = true;
  error.value = null;
  
  try {
    const data = await invoke<StatisticsData>('get_statistics_data', {
      apiConfig: props.apiConfig
    });
    statisticsData.value = data;
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Unknown error occurred';
    console.error('Failed to load statistics:', err);
    // Set some default data to prevent crashes
    statisticsData.value = {
      trends: [],
      charts: [],
      insights: [],
      programmer_class: {
        class_name: "Future Coder",
        description: "Ready to embark on an exciting journey into the world of programming.",
        technologies: ["HTML", "CSS", "JavaScript"],
        level: "Beginner",
        color: "#607D8B"
      }
    };
  } finally {
    isLoading.value = false;
  }
};

onMounted(() => {
  loadStatistics();
});
</script>
