<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  Chart as ChartJS,
  Title,
  Tooltip,
  Legend,
  BarElement,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  ChartOptions,
} from 'chart.js'
import annotationPlugin from 'chartjs-plugin-annotation';

import { Line } from 'vue-chartjs'

// ChartJS.register(CategoryScale, LinearScale, BarElement, Title, Tooltip, Legend)
ChartJS.register(
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend,
  annotationPlugin
)

interface RssiPoint {
  id: number;
  peak: number;
  time: number;
  duration: number;
  race_id: number;
}

const passHigh = ref(90)
const passLow = ref(80)
const laps = ref<{ time: number }[]>([]);

const lapSplits = computed(() => {
  if (laps.value.length === 0) {
    return [];
  }
  const splits = [laps.value[0].time];
  for (let i = 1; i < laps.value.length; i++) {
    splits.push(laps.value[i].time - laps.value[i - 1].time);
  }
  return splits;
});
const rssi = ref<RssiPoint[]>([]);
const dataVersion = ref(0);
const rawData = ref([
  { time: 1.23332, peak: 23 },
  { time: 1.5435435, peak: 45 },
  { time: 3.435345, peak: 67 },
  { time: 4.23332, peak: 12 },
  { time: 5.23332, peak: 32 },
]);

const chartData = computed(() => rawData.value.map(point => ({
  x: point.time, // Conversion des microsecondes en secondes
  y: point.peak
})));


const chartDatasets = computed(() => [{
  label: 'Data One',
  backgroundColor: '#24c8db',
  borderColor: '#24c8db',
  borderWidth: 2,
  pointRadius: 0,
  tension: 0.1,
  data: chartData.value,
}]);

const chartSetup = computed(() => ({
  datasets: chartDatasets.value
}));


const options = computed((): ChartOptions<'line'> => ({
  scales: {
    x: {
      type: 'linear' as const, // Axe X numérique
      title: {
        display: true,
        text: 'Temps (en secondes depuis le début)'
      }
    },
    y: {
      title: {
        display: true,
        text: 'RSSI'
      },
      beginAtZero: true
    }
  },
  plugins: {
    annotation: {
      annotations: {
        line1: {
          type: 'line' as const,
          yMin: passHigh.value,
          yMax: passHigh.value,
          borderColor: 'rgb(255, 99, 132)',
          borderWidth: 2,
        },
        line2: {
          type: 'line' as const,
          yMin: passLow.value,
          yMax: passLow.value,
          borderColor: 'rgb(255, 99, 132)',
          borderWidth: 2,
        },
        ...Object.fromEntries(laps.value.map((lap, index) => [
          `lap-${index}`,
          {
            type: 'line' as const,
            xMin: lap.time,
            xMax: lap.time,
            borderColor: 'rgb(54, 162, 235)',
            borderWidth: 2,
            label: {
              content: `Lap ${index + 1}`,
              display: true,
            },
          }
        ]))
      }
    }
  }
}))

const greetMsg = ref("");
const name = ref("");

async function getRSSI() {
  console.log("getRSSI");
  rssi.value = await invoke("get_rssi", {});
  rawData.value = rssi.value.flatMap(point => [
    { time: point.time, peak: point.peak },
    { time: point.time + point.duration, peak: point.peak }
  ]);
  dataVersion.value++;

  computeLap()
}



function computeLap() {
  const tempLaps: { time: number }[] = [];
  if (rssi.value.length === 0) {
    laps.value = tempLaps;
    return;
  }

  let state = 'BELOW_LOW'; // Possible states: BELOW_LOW, ABOVE_LOW, IN_LAP
  let startTime = 0;
  let lapConfirmed = false;

  // Sort rssi data by time to ensure chronological processing
  const sortedRssi = [...rssi.value].sort((a, b) => a.time - b.time);

  for (const point of sortedRssi) {
    const currentTime = point.time;
    const currentPeak = point.peak;

    switch (state) {
      case 'BELOW_LOW':
        if (currentPeak > passLow.value) {
          state = 'ABOVE_LOW';
          startTime = currentTime;
          if (currentPeak > passHigh.value) {
            state = 'IN_LAP';
            lapConfirmed = true;
          }
        }
        break;

      case 'ABOVE_LOW':
        if (currentPeak > passHigh.value) {
          state = 'IN_LAP';
          lapConfirmed = true;
        } else if (currentPeak <= passLow.value) {
          state = 'BELOW_LOW';
          // Reset if it drops below low without confirming a lap
          lapConfirmed = false;
        }
        break;

      case 'IN_LAP':
        if (currentPeak <= passLow.value) {
          if (lapConfirmed) {
            const endTime = currentTime;
            const lapTime = startTime + (endTime - startTime) / 2;
            tempLaps.push({ time: lapTime });
          }
          state = 'BELOW_LOW';
          lapConfirmed = false;
        }
        break;
    }
  }

  laps.value = tempLaps;
}



async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  greetMsg.value = await invoke("greet", { name: name.value });
}
</script>

<template>
  <main class="container">
    <h1>Welcome to Tauri + Vue</h1>
    <Line :data="chartSetup" :options="options" :key="dataVersion" />
     <ul v-if="laps.length > 0">
      <li v-for="(lap, index) in laps" :key="lap.time">
        Lap {{ index + 1 }}: {{ lap.time.toFixed(3) }} (Split: {{ lapSplits[index].toFixed(3) }})
      </li>
    </ul>
    <button @click="getRSSI">Get RSSI</button>
    <div class="row">
      <a href="https://vite.dev" target="_blank">
        <img src="/vite.svg" class="logo vite" alt="Vite logo" />
      </a>
      <a href="https://tauri.app" target="_blank">
        <img src="/tauri.svg" class="logo tauri" alt="Tauri logo" />
      </a>
      <a href="https://vuejs.org/" target="_blank">
        <img src="./assets/vue.svg" class="logo vue" alt="Vue logo" />
      </a>
    </div>
    <p>Click on the Tauri, Vite, and Vue logos to learn more.</p>

    <form class="row" @submit.prevent="greet">
      <input id="greet-input" v-model="name" placeholder="Enter a name..." />
      <button type="submit">Greet</button>
    </form>
    <p>{{ greetMsg }}</p>
  </main>
</template>

<style scoped>
.logo.vite:hover {
  filter: drop-shadow(0 0 2em #747bff);
}

.logo.vue:hover {
  filter: drop-shadow(0 0 2em #249b73);
}
</style>
<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: 0.75s;
}

.logo.tauri:hover {
  filter: drop-shadow(0 0 2em #24c8db);
}

.row {
  display: flex;
  justify-content: center;
}

a {
  font-weight: 500;
  color: #646cff;
  text-decoration: inherit;
}

a:hover {
  color: #535bf2;
}

h1 {
  text-align: center;
}

input,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}

button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

input,
button {
  outline: none;
}

#greet-input {
  margin-right: 5px;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  a:hover {
    color: #24c8db;
  }

  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }

  button:active {
    background-color: #0f0f0f69;
  }
}
</style>