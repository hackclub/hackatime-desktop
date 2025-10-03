<template>
  <div class="bg-bg-card border border-border-primary rounded-2xl p-6 shadow-card">
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-lg font-semibold text-text-primary">{{ title }}</h3>
      <div class="text-sm text-text-secondary">{{ period }}</div>
    </div>
    
    <div class="h-64">
      <canvas ref="chartCanvas"></canvas>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  BarElement,
  ArcElement,
  Title,
  Tooltip,
  Legend,
  Filler,
  BarController,
  LineController,
  DoughnutController,
  PieController
} from 'chart.js';

// Register Chart.js components
ChartJS.register(
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  BarElement,
  ArcElement,
  Title,
  Tooltip,
  Legend,
  Filler,
  BarController,
  LineController,
  DoughnutController,
  PieController
);

interface Props {
  title: string;
  chartType: string;
  data: any;
  period: string;
  colorScheme: string;
}

const props = defineProps<Props>();
const chartCanvas = ref<HTMLCanvasElement | null>(null);
let chartInstance: ChartJS | null = null;

const createChart = () => {
  if (!chartCanvas.value) return;

  // Destroy existing chart
  if (chartInstance) {
    chartInstance.destroy();
  }

  const ctx = chartCanvas.value.getContext('2d');
  if (!ctx) return;

  const config = {
    type: props.chartType,
    data: props.data,
    options: {
      responsive: true,
      maintainAspectRatio: false,
      plugins: {
        legend: {
          display: props.chartType === 'doughnut' || props.chartType === 'pie',
          position: 'bottom' as const,
          labels: {
            color: '#B0BAC4',
            font: {
              family: 'Inter, system-ui, sans-serif'
            }
          }
        },
        tooltip: {
          backgroundColor: '#191415',
          titleColor: '#FFFFFF',
          bodyColor: '#B0BAC4',
          borderColor: '#FB4B20',
          borderWidth: 1,
          cornerRadius: 8,
          displayColors: true
        }
      },
      scales: props.chartType !== 'doughnut' && props.chartType !== 'pie' ? {
        x: {
          grid: {
            color: '#2A2A2A',
            drawBorder: false
          },
          ticks: {
            color: '#B0BAC4',
            font: {
              family: 'Inter, system-ui, sans-serif'
            }
          }
        },
        y: {
          grid: {
            color: '#2A2A2A',
            drawBorder: false
          },
          ticks: {
            color: '#B0BAC4',
            font: {
              family: 'Inter, system-ui, sans-serif'
            }
          }
        }
      } : undefined
    }
  };

  chartInstance = new ChartJS(ctx, config);
};

onMounted(() => {
  createChart();
});

onUnmounted(() => {
  if (chartInstance) {
    chartInstance.destroy();
  }
});

watch(() => props.data, () => {
  createChart();
}, { deep: true });
</script>
