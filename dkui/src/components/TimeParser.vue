<script setup>
import { ref, watch, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

const commonTimezones = [
  { label: "UTC+9 (Japan, Korea)", value: "+09:00" },
  { label: "UTC+8 (China, Singapore)", value: "+08:00" },
  { label: "UTC+7 (Thailand, Vietnam)", value: "+07:00" },
  { label: "UTC+1 (Paris, Berlin)", value: "+01:00" },
  { label: "UTC+0 (London)", value: "+00:00" },
  { label: "UTC-5 (New York, Toronto)", value: "-05:00" },
  { label: "UTC-8 (Los Angeles, Vancouver)", value: "-08:00" },
];
const fmts = [
  "%Y-%m-%d %H:%M:%S%z",
  "%Y-%m-%d %H:%M:%S",
  "%Y/%m/%d %H:%M:%S",
  "rfc3339",
  "ts",
];
const timeOutput = ref({
  input: "",
  timestamp: 0,
  output: "",
});
const timeInput = ref("");
const timeTimezone = ref("");
const timeFormat = ref(fmts[0]);
const timeFormatType = ref("FMT"); // FMT, Timestamp
const timeUnit = ref("ms");

const isUpdating = ref(false);
let updateInterval = null;

let debounceTimer = null;
function debounce(fn, delay = 300) {
  return (...args) => {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => fn(...args), delay);
  };
}

async function getNow() {
  timeInput.value = `${new Date().getTime()}`;
  parseTime();
}

function startAutoUpdate() {
  isUpdating.value = true;
  getNow(); // 立即更新一次
  updateInterval = setInterval(getNow, 333);
}

function stopAutoUpdate() {
  isUpdating.value = false;
  if (updateInterval) {
    clearInterval(updateInterval);
    updateInterval = null;
  }
}

function toggleAutoUpdate() {
  if (isUpdating.value) {
    stopAutoUpdate();
  } else {
    startAutoUpdate();
  }
}

onUnmounted(() => {
  stopAutoUpdate();
});

async function parseTime() {
  if (!timeInput.value) {
    timeOutput.value = "";
    return;
  }
  const format = timeFormatType.value === "Timestamp" ? "ts" : (timeFormat.value || null);
  try {

    let result = await invoke("parse_time", {
      time: timeInput.value,
      unit: timeUnit.value,
      timezone: timeTimezone.value || null,
      format: format
    });
    timeOutput.value = JSON.parse(result);
  } catch (e) {
    timeOutput.value = "Error: " + e;
  }
}

async function copyToClipboard(e) {
  const text = e.target.innerText;
  try {
    await navigator.clipboard.writeText(text);
    // Visual feedback
    const originalBg = e.target.style.backgroundColor;
    e.target.style.backgroundColor = '#d4edda';
    setTimeout(() => {
      e.target.style.backgroundColor = originalBg;
    }, 500);
  } catch (err) {
    console.error('Failed to copy: ', err);
  }
}

function handleTimezoneKeydown(e) {
  if (e.key !== 'ArrowUp' && e.key !== 'ArrowDown') return;
  
  e.preventDefault();
  
  let current = timeTimezone.value || "+00:00";
  // Handle common formats like UTC+8, +08:00, or just 8
  let match = current.match(/([+-]?\d+)(?::(\d+))?/);
  
  let hours = 0;
  let minutes = 0;
  
  if (match) {
    hours = parseInt(match[1]);
    minutes = match[2] ? parseInt(match[2]) : 0;
  } else if (current.toUpperCase() === 'UTC') {
    hours = 0;
    minutes = 0;
  } else {
    // If we can't parse it, start from 0
    hours = 0;
    minutes = 0;
  }

  if (e.key === 'ArrowUp') {
    hours++;
  } else {
    hours--;
  }
  
  // Clamp hours between -12 and +14 (common timezone range)
  if (hours > 14) hours = 14;
  if (hours < -12) hours = -12;
  
  const sign = hours >= 0 ? '+' : '-';
  const absHours = Math.abs(hours).toString().padStart(2, '0');
  const absMinutes = minutes.toString().padStart(2, '0');
  
  timeTimezone.value = `${sign}${absHours}:${absMinutes}`;
}

watch([timeInput, timeTimezone, timeFormat, timeUnit, timeFormatType], debounce(() => {
  if (timeInput.value) {
    parseTime();
  }
}));
</script>

<template>
  <section class="tool-section">
    <div class="row">
      <input 
        v-model="timeInput" 
        placeholder="Enter time string or timestamp..." 
        :disabled="isUpdating"
        :class="{ 'is-updating': isUpdating }"
      />
      <div class="timezone-input-container">
        <input 
          v-model="timeTimezone" 
          class="timezone-select" 
          list="common-timezones"
          placeholder="Local Timezone"
          @keydown="handleTimezoneKeydown"
        />
        <datalist id="common-timezones">
          <option v-for="tz in commonTimezones" :key="tz.value" :value="tz.value">
            {{ tz.label }}
          </option>
        </datalist>
      </div>
      <button 
        class="refresh-button" 
        :class="{ active: isUpdating }"
        @click="toggleAutoUpdate" 
        :title="isUpdating ? 'Stop Auto Update' : 'Get Current Time'"
      >
        <template v-if="!isUpdating">
          Now
        </template>
        <svg 
          v-else
          :class="{ 'spinning': isUpdating }"
          viewBox="0 0 24 24" 
          width="16" 
          height="16" 
          fill="none" 
          stroke="currentColor" 
          stroke-width="2" 
          stroke-linecap="round" 
          stroke-linejoin="round"
        >
          <path d="M23 4v6h-6"></path>
          <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"></path>
        </svg>
      </button>
    </div>
    <div class="row">
      <div class="format-input-group">
        <div class="button-group">
          <button 
            :class="{ active: timeFormatType === 'Timestamp' }" 
            @click="timeFormatType = 'Timestamp'"
          >
            Timestamp
          </button>
          <button 
            :class="{ active: timeFormatType === 'FMT' }" 
            @click="timeFormatType = 'FMT'"
          >
            FMT
          </button>
        </div>
        <input 
          v-if="timeFormatType === 'FMT'"
          v-model="timeFormat" 
          list="common-fmts"
          placeholder="Format (rfc3339, ts, or custom %Y-%m-%d...)" 
        />
        <datalist id="common-fmts">
          <option v-for="f in fmts" :key="f" :value="f" />
        </datalist>
      </div>
    </div>
    <div v-if="timeOutput" class="output-container">
      <div class="copy-tip">Double Click to Copy</div>
      Formated: <pre class="output" @dblclick="copyToClipboard">{{ timeOutput.output }}</pre>
    </div>
    <div v-if="timeOutput" class="output-container">
      <div class="copy-tip">Double Click to Copy</div>
      Timestamp(ms):<pre class="output" @dblclick="copyToClipboard">{{ timeOutput.timestamp }}</pre>
    </div>
  </section>
</template>

<style scoped>
.tool-section {
  margin-bottom: 30px;
  padding: 20px;
  border: 1px solid #ccc;
  border-radius: 8px;
  text-align: left;
}

.row {
  display: flex;
  gap: 10px;
  margin-bottom: 10px;
  align-items: center;
}

.format-input-group {
  flex: 1;
  display: flex;
  gap: 0;
}

.format-input-group input {
  border-top-left-radius: 0;
  border-bottom-left-radius: 0;
  border-left: none;
}

.button-group {
  display: flex;
  border: 1px solid #ddd;
  border-radius: 4px;
  overflow: hidden;
}

.format-input-group .button-group {
  border-top-right-radius: 0;
  border-bottom-right-radius: 0;
  border-right: none;
}

button {
  padding: 8px 16px;
  background-color: #396cd8;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.refresh-button {
  padding: 8px 12px;
  width: 64px;
  height: 36px;
  flex-shrink: 0;
  background-color: white;
  color: #396cd8;
  border: 1px solid #ddd;
  font-size: 14px;
}

.refresh-button.active {
  background-color: #396cd8;
  color: white;
  border-color: #396cd8;
}

.refresh-button:hover:not(.active) {
  background-color: #f5f5f5;
}

.refresh-button:hover.active {
  background-color: #2a52a8;
}

.spinning {
  animation: spin 2s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.button-group button {
  background-color: white;
  color: black;
  border-radius: 0;
  padding: 8px 12px;
  font-size: 14px;
  transition: all 0.2s;
  height: auto;
  width: auto;
}

.button-group button:not(:last-child) {
  border-right: 1px solid #ddd;
}

.button-group button.active {
  background-color: #007aff;
  color: white;
}

.button-group button:hover:not(.active) {
  background-color: #f5f5f5;
}

button:hover {
  background-color: #2a52a8;
}

.output-container {
  position: relative;
  margin-top: 10px;
}

.copy-tip {
  position: absolute;
  top: 5px;
  right: 10px;
  font-size: 10px;
  color: #888;
  pointer-events: none;
  opacity: 0;
  transition: opacity 0.2s;
}

.output-container:hover .copy-tip {
  opacity: 0.7;
}

.output {
  margin-top: 0;
  padding: 10px;
  background-color: #f0f0f0;
  border-radius: 4px;
  white-space: pre-wrap;
  word-break: break-all;
  cursor: pointer;
  transition: background-color 0.2s;
}

.output:hover {
  background-color: #e8e8e8;
}

.unit-select {
  flex: 0 0 140px;
}

.timezone-input-container {
  flex: 0 0 160px;
  display: flex;
}

input, select {
  flex: 1;
  padding: 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background-color: white;
  color: black;
  box-sizing: border-box;
}

input.is-updating {
  background-color: #f8f9fa;
  border-color: #e9ecef;
  color: #6c757d;
  cursor: not-allowed;
}

@media (prefers-color-scheme: dark) {
  .output {
    background-color: #1e1e1e;
    color: #d4d4d4;
  }
  .output:hover {
    background-color: #2a2a2a;
  }
  .tool-section {
    border-color: #444;
  }
  .refresh-button {
    background-color: #2a2a2a;
    border-color: #444;
    color: #007aff;
  }
  .refresh-button.active {
    background-color: #007aff;
    color: white;
    border-color: #007aff;
  }
  .refresh-button:hover:not(.active) {
    background-color: #3e3e3e;
  }
  .refresh-button:hover.active {
    background-color: #005bb7;
  }
  input, select, .button-group {
    background-color: #2a2a2a;
    border-color: #444;
    color: #d4d4d4;
  }
  input.is-updating {
    background-color: #1a1a1a;
    border-color: #333;
    color: #888;
  }
  .button-group button {
    background-color: #2a2a2a;
    color: #d4d4d4;
  }
  .button-group button:not(:last-child) {
    border-right-color: #444;
  }
  .button-group button.active {
    background-color: #007aff;
    color: white;
  }
  .button-group button:hover:not(.active) {
    background-color: #3a3a3a;
  }
}
</style>
