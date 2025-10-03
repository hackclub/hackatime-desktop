<template>
  <div class="h-32">
    <canvas ref="chartCanvas"></canvas>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  BarElement,
  BarController,
  Tooltip,
  Legend
} from 'chart.js';

// Register Chart.js components
ChartJS.register(
  CategoryScale,
  LinearScale,
  BarElement,
  BarController,
  Tooltip,
  Legend
);

interface Props {
  data: Array<{
    date: string;
    day_name: string;
    hours: number;
    percentage: number;
  }>;
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

  // Prepare data for Chart.js
  const labels = props.data.map(day => day.day_name);
  const chartData = props.data.map(day => day.hours);
  
  // Calculate colors for each bar
  const maxHours = Math.max(...chartData, 1);
  const colors = chartData.map(hours => {
    if (hours === 0) return '#3d2b2e';
    
    const intensity = hours / maxHours;
    const startColor = { r: 237, g: 141, b: 75 };    // #ED8D4B (lighter)
    const endColor = { r: 251, g: 75, b: 32 };       // #FB4B20 (darker)
    
    const r = Math.round(startColor.r + (endColor.r - startColor.r) * intensity);
    const g = Math.round(startColor.g + (endColor.g - startColor.g) * intensity);
    const b = Math.round(startColor.b + (endColor.b - startColor.b) * intensity);
    
    return `rgb(${r}, ${g}, ${b})`;
  });

  const config = {
    type: 'bar' as const,
    data: {
      labels: labels,
      datasets: [{
        data: chartData,
        backgroundColor: colors,
        borderColor: colors,
        borderWidth: 0,
        borderRadius: 4,
        borderSkipped: false,
      }]
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      plugins: {
        legend: {
          display: false
        },
        tooltip: {
          backgroundColor: '#191415',
          titleColor: '#FFFFFF',
          bodyColor: '#B0BAC4',
          borderColor: '#FB4B20',
          borderWidth: 1,
          cornerRadius: 8,
          displayColors: false,
          callbacks: {
            label: function(context: any) {
              return `${context.parsed.y}h`;
            }
          }
        }
      },
      scales: {
        x: {
          display: true,
          grid: {
            display: false
          },
          ticks: {
            color: '#B0BAC4',
            font: {
              size: 10,
              family: 'Inter, system-ui, sans-serif'
            }
          }
        },
        y: {
          display: false
        }
      }
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
