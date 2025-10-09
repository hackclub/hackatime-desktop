<template>
  <div v-if="statisticsData" class="space-y-8">
    <!-- Trends Section -->
    <div>
      <h2 class="text-xl font-semibold text-text-primary mb-8">Trends</h2>
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mt-4">
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
      <h2 class="text-xl font-semibold text-text-primary mb-8">Analytics</h2>
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mt-4">
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
      <h2 class="text-xl font-semibold text-text-primary mb-8">Your Programmer Class</h2>
      <div class="card-3d mt-4">
        <div class="rounded-[8px] border border-black p-6 card-3d-front" style="background-color: #3D2C3E;">
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
            class="px-3 py-1 bg-[rgba(255,255,255,0.06)] text-text-primary rounded-lg text-sm font-medium"
          >
            {{ tech }}
          </span>
        </div>
        </div>
      </div>
    </div>

    <!-- Insights Section -->
    <div>
      <h2 class="text-xl font-semibold text-text-primary mb-8">Insights</h2>
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 mt-4">
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
  <RandomLoader v-else-if="isLoading" />

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
import RandomLoader from './RandomLoader.vue';

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

<style scoped>
.card-3d { position: relative; border-radius: 8px; padding: 0; }
.card-3d::before { content: ''; position: absolute; inset: 0; border-radius: 8px; background: #2A1F2B; z-index: 0; }
.card-3d-front { position: relative; transform: translateY(-6px); z-index: 1; }
</style>
