<template>
  <div class="card-3d">
    <div class="rounded-[8px] border-2 border-black p-6 card-3d-front h-full flex flex-col" style="background-color: #3D2C3E;">
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-lg font-semibold text-text-primary">{{ title }}</h3>
      <div class="text-sm text-text-secondary">{{ period }}</div>
    </div>
    
    <div class="h-64 flex-1">
      <canvas ref="chartCanvas"></canvas>
    </div>
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

<style scoped>
.card-3d { position: relative; border-radius: 8px; padding: 0; }
.card-3d::before { content: ''; position: absolute; inset: 0; border-radius: 8px; background: #2A1F2B; z-index: 0; }
.card-3d-front { position: relative; transform: translateY(-6px); z-index: 1; }
</style>
