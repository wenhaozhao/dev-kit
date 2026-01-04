<script setup>
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";

const timeOutput = ref("");
const timeInput = ref("");
const timeTimezone = ref("");
const timeFormat = ref("");
const timeUnit = ref("ms");

let debounceTimer = null;
function debounce(fn, delay = 300) {
  return (...args) => {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => fn(...args), delay);
  };
}

async function getNow() {
  try {
    timeInput.value = await invoke("now_time", {
      timezone: timeTimezone.value || null,
      format: "ts",
      unit: timeUnit.value
    });
    
    timeOutput.value = await invoke("now_time", { 
      timezone: timeTimezone.value || null, 
      format: timeFormat.value || null,
      unit: timeUnit.value
    });
  } catch (e) {
    timeOutput.value = "Error: " + e;
  }
}

async function parseTime() {
  if (!timeInput.value) {
    timeOutput.value = "";
    return;
  }
  try {
    timeOutput.value = await invoke("parse_time", {
      time: timeInput.value,
      unit: timeUnit.value,
      timezone: timeTimezone.value || null,
      format: timeFormat.value || null
    });
  } catch (e) {
    timeOutput.value = "Error: " + e;
  }
}

watch([timeInput, timeTimezone, timeFormat, timeUnit], debounce(() => {
  if (timeInput.value) {
    parseTime();
  }
}));
</script>

<template>
  <section class="tool-section">
    <div class="row">
      <input v-model="timeInput" placeholder="Enter time string or timestamp..." />
      <select v-model="timeUnit" class="unit-select">
        <option value="s">Seconds (s)</option>
        <option value="ms">Milliseconds (ms)</option>
      </select>
      <button @click="getNow">Get Current Time</button>
    </div>
    <div class="row">
      <input v-model="timeTimezone" placeholder="Timezone (e.g. +08:00)" />
      <input v-model="timeFormat" placeholder="Format (rfc3339, ts, or custom %Y-%m-%d...)" />
    </div>
    <pre v-if="timeOutput" class="output">{{ timeOutput }}</pre>
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
}

.unit-select {
  flex: 0 0 150px;
}

input, select {
  flex: 1;
  padding: 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background-color: white;
  color: black;
}

button {
  padding: 8px 16px;
  background-color: #396cd8;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

button:hover {
  background-color: #2a52a8;
}

.output {
  margin-top: 10px;
  padding: 10px;
  background-color: #f0f0f0;
  border-radius: 4px;
  white-space: pre-wrap;
  word-break: break-all;
}

@media (prefers-color-scheme: dark) {
  .output {
    background-color: #1e1e1e;
    color: #d4d4d4;
  }
  .tool-section {
    border-color: #444;
  }
  input, select {
    background-color: #2a2a2a;
    border-color: #444;
    color: #d4d4d4;
  }
}
</style>
