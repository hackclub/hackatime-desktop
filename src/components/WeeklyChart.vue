<template>
  <div class="w-full h-40 overflow-hidden">
    <canvas ref="chartCanvas" class="w-full h-full"></canvas>
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

  
  if (chartInstance) {
    chartInstance.destroy();
  }

  const ctx = chartCanvas.value.getContext('2d');
  if (!ctx) return;

  
  const labels = props.data.map(day => day.day_name);
  const chartData = props.data.map(day => day.hours);
  
  
  const maxHours = Math.max(...chartData, 0);
  const minHours = Math.min(...chartData, 0);
  
  const mostColor = { r: 232, g: 131, b: 174 };   
  const leastColor = { r: 233, g: 150, b: 130 };  

  const colors = chartData.map(hours => {
    
    const t = maxHours === minHours ? 0.5 : (hours - minHours) / (Math.max(maxHours - minHours, 1e-6));
    const r = Math.round(leastColor.r + (mostColor.r - leastColor.r) * t);
    const g = Math.round(leastColor.g + (mostColor.g - leastColor.g) * t);
    const b = Math.round(leastColor.b + (mostColor.b - leastColor.b) * t);
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
        borderRadius: 6,
        borderSkipped: false,
      }]
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      resizeDelay: 0,
      layout: {
        padding: {
          top: 10,
          bottom: 28,
          left: 10,
          right: 10
        }
      },
      plugins: {
        legend: {
          display: false
        },
        tooltip: {
          backgroundColor: '#1F1617',
          titleColor: '#FFFFFF',
          bodyColor: '#F5E6E8',
          borderColor: '#2A1F2B',
          borderWidth: 1,
          cornerRadius: 8,
          displayColors: false,
          padding: 10,
          callbacks: {
            label: function(context: any) {
              return `${context.parsed.y.toFixed(1)}h`;
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
            color: '#F5E6E8',
            font: {
              size: 11,
              family: 'Outfit, system-ui, sans-serif',
              weight: 500
            },
            padding: 0
          }
        },
        y: {
          display: true,
          beginAtZero: true,
          suggestedMin: 0,
          grace: '10%',
          grid: {
            color: '#2A1F2B',
            drawBorder: false,
            display: false
          },
          border: {
            display: false
          },
          ticks: {
            color: '#F5E6E8',
            font: {
              size: 10,
              family: 'Outfit, system-ui, sans-serif'
            },
            padding: 4,
            callback: function(value: any) {
              return value + 'h';
            }
          }
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
