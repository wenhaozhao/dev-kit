<script setup>
import { ref, onMounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";

const uriInput = ref("");
const uriOutput = ref("");
const jsonInput = ref("");
const jsonRightInput = ref("");
const jsonOutput = ref("");
const jsonRightOutput = ref("");
const jsonQuery = ref("");
const diffTool = ref("");
const availableDiffTools = ref([]);
const timeOutput = ref("");
const timeInput = ref("");
const timeTimezone = ref("");
const timeFormat = ref("");
const timeUnit = ref("ms");
const jsonKeys = ref([]);
const selectedIndex = ref(-1);

const currentTab = ref("json");

const leftTextarea = ref(null);
const rightTextarea = ref(null);
let isSyncing = false;

let debounceTimer = null;
function debounce(fn, delay = 300) {
  return (...args) => {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => fn(...args), delay);
  };
}

onMounted(async () => {
  try {
    availableDiffTools.value = await invoke("get_available_diff_tools");
    if (availableDiffTools.value.length > 0) {
      diffTool.value = availableDiffTools.value[0];
    }
  } catch (e) {
    console.error("Failed to fetch diff tools:", e);
  }

  // Sync textarea heights in JSON Diff
  let observer = null;
  const syncHeight = (entries) => {
    if (isSyncing) return;
    isSyncing = true;
    const height = entries[0].contentRect.height;
    if (leftTextarea.value && rightTextarea.value) {
      leftTextarea.value.style.height = `${height}px`;
      rightTextarea.value.style.height = `${height}px`;
    }
    setTimeout(() => { isSyncing = false; }, 0);
  };

  watch(currentTab, async (newTab) => {
    if (newTab === 'diff') {
      // Wait for DOM to be updated by v-if
      setTimeout(() => {
        if (!observer) {
          observer = new ResizeObserver(syncHeight);
        }
        if (leftTextarea.value) observer.observe(leftTextarea.value);
        if (rightTextarea.value) observer.observe(rightTextarea.value);
      }, 0);
    } else {
      if (observer) {
        observer.disconnect();
      }
    }
  }, { immediate: true });
});

async function decodeUri() {
  try {
    uriOutput.value = await invoke("decode_uri", { uri: uriInput.value });
  } catch (e) {
    uriOutput.value = "Error: " + e;
  }
}

async function formatJson(input, outputRef) {
  try {
    outputRef.value = await invoke("format_json", { json: input });
  } catch (e) {
    outputRef.value = "Error: " + e;
  }
}

const lastSuccessfulOutput = ref("");
const lastSuccessfulRightOutput = ref("");

async function queryJson() {
  if (!jsonInput.value && !jsonRightInput.value) {
    jsonOutput.value = "";
    jsonRightOutput.value = "";
    jsonKeys.value = [];
    lastSuccessfulOutput.value = "";
    lastSuccessfulRightOutput.value = "";
    return;
  }
  
  // Auto-prefix '$' if missing
  if (jsonQuery.value && !jsonQuery.value.startsWith('$')) {
    jsonQuery.value = '$' + (jsonQuery.value.startsWith('.') ? '' : '.') + jsonQuery.value;
  }

  const query = (!jsonQuery.value || jsonQuery.value === '$') ? null : jsonQuery.value;

  if (!query) {
    if (jsonInput.value) {
      await formatJson(jsonInput.value, jsonOutput);
      lastSuccessfulOutput.value = jsonOutput.value;
    } else {
      jsonOutput.value = "";
    }
    if (jsonRightInput.value) {
      await formatJson(jsonRightInput.value, jsonRightOutput);
      lastSuccessfulRightOutput.value = jsonRightOutput.value;
    } else {
      jsonRightOutput.value = "";
    }
    await updateKeys();
    return;
  }

  // Handle Left Input
  if (jsonInput.value) {
    try {
      const res = await invoke("query_json", { json: jsonInput.value, query });
      jsonOutput.value = res.join("\n");
      lastSuccessfulOutput.value = jsonOutput.value;
    } catch (e) {
      if (lastSuccessfulOutput.value) {
        jsonOutput.value = lastSuccessfulOutput.value;
      } else {
        jsonOutput.value = "Error: " + e;
      }
    }
  } else {
    jsonOutput.value = "";
  }

  // Handle Right Input
  if (jsonRightInput.value) {
    try {
      const res = await invoke("query_json", { json: jsonRightInput.value, query });
      jsonRightOutput.value = res.join("\n");
      lastSuccessfulRightOutput.value = jsonRightOutput.value;
    } catch (e) {
      if (lastSuccessfulRightOutput.value) {
        jsonRightOutput.value = lastSuccessfulRightOutput.value;
      } else {
        jsonRightOutput.value = "Error: " + e;
      }
    }
  } else {
    jsonRightOutput.value = "";
  }

  await updateKeys();
}

async function updateKeys() {
  try {
    const jsonToProcess = jsonInput.value || jsonRightInput.value;
    if (!jsonToProcess) {
      jsonKeys.value = [];
      return;
    }
    jsonKeys.value = await invoke("get_json_keys", { 
      json: jsonToProcess, 
      query: jsonQuery.value || null 
    });
  } catch (e) {
    console.error("Failed to fetch keys:", e);
    jsonKeys.value = [];
  }
}

const showSuggestions = ref(false);

function appendToQuery(key) {
  if (!jsonQuery.value) {
    jsonQuery.value = "$.";
  }
  if (jsonQuery.value.endsWith(".")) {
    jsonQuery.value += key;
  } else {
    jsonQuery.value += "." + key;
  }
  showSuggestions.value = false;
  selectedIndex.value = -1;
}

function handleKeyDown(e) {
  if (!showSuggestions.value || jsonKeys.value.length === 0) return;

  if (e.key === 'ArrowDown') {
    e.preventDefault();
    selectedIndex.value = (selectedIndex.value + 1) % jsonKeys.value.length;
  } else if (e.key === 'ArrowUp') {
    e.preventDefault();
    selectedIndex.value = (selectedIndex.value - 1 + jsonKeys.value.length) % jsonKeys.value.length;
  } else if (e.key === 'Enter') {
    if (selectedIndex.value >= 0) {
      e.preventDefault();
      appendToQuery(jsonKeys.value[selectedIndex.value]);
    }
  } else if (e.key === 'Escape') {
    showSuggestions.value = false;
    selectedIndex.value = -1;
  }
}

watch(jsonKeys, () => {
  selectedIndex.value = -1;
});

watch([jsonInput, jsonRightInput, jsonQuery], debounce(() => {
  if (currentTab.value === 'json' || currentTab.value === 'diff') {
     queryJson();
  }
}));

watch(uriInput, debounce(() => {
  if (currentTab.value === 'uri') {
    decodeUri();
  }
}));

async function diffJson() {
  try {
    await invoke("diff_json", { 
      left: jsonInput.value, 
      right: jsonRightInput.value, 
      query: jsonQuery.value || null,
      diffTool: diffTool.value 
    });
  } catch (e) {
    jsonOutput.value = "Error: " + e;
  }
}

async function getNow() {
  try {
    // Fill timeInput with current timestamp
    timeInput.value = await invoke("now_time", {
      timezone: timeTimezone.value || null,
      format: "ts",
      unit: timeUnit.value
    });
    
    // Also update timeOutput with the requested format
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
  if (currentTab.value === 'time' && timeInput.value) {
    parseTime();
  }
}));

async function saveJsonToFile() {
  if (!jsonOutput.value || jsonOutput.value.startsWith("Error: ")) {
    return;
  }
  try {
    const path = await save({
      filters: [{
        name: 'JSON',
        extensions: ['json']
      }]
    });
    if (path) {
      await invoke("save_to_file", { path, content: jsonOutput.value });
    }
  } catch (e) {
    jsonOutput.value = "Error: " + e;
  }
}
</script>

<template>
  <main class="container">
    <div class="tabs">
      <button :class="{ active: currentTab === 'json' }" @click="currentTab = 'json'">JSON Parser</button>
      <button :class="{ active: currentTab === 'diff' }" @click="currentTab = 'diff'">JSON Diff</button>
      <button :class="{ active: currentTab === 'uri' }" @click="currentTab = 'uri'">URI Tools</button>
      <button :class="{ active: currentTab === 'time' }" @click="currentTab = 'time'">Time Tools</button>
    </div>

    <section v-if="currentTab === 'uri'" class="tool-section">
      <div class="row">
        <input v-model="uriInput" placeholder="Enter URI to decode..." />
      </div>
      <pre v-if="uriOutput" class="output">{{ uriOutput }}</pre>
    </section>

    <section v-if="currentTab === 'json'" class="tool-section">
      <div class="json-inputs">
        <textarea v-model="jsonInput" placeholder="Enter JSON..." rows="10"></textarea>
      </div>
      <div class="row query-row">
        <input v-model="jsonQuery" placeholder="json path filter" 
          @input="showSuggestions = true"
          @blur="setTimeout(() => showSuggestions = false, 200)"
          @keydown="handleKeyDown" />
        <button v-if="jsonOutput && !jsonOutput.startsWith('Error: ')" @click="saveJsonToFile">Save</button>
        <div v-if="showSuggestions && jsonKeys.length > 0" class="suggestions-dropdown">
          <div v-for="(key, index) in jsonKeys" :key="key" 
            class="suggestion-item" 
            :class="{ active: index === selectedIndex }"
            @mousedown.prevent="appendToQuery(key)">
            {{ key }}
          </div>
        </div>
      </div>
      <div class="json-outputs">
        <pre v-if="jsonOutput" class="output">{{ jsonOutput }}</pre>
      </div>
    </section>

    <section v-if="currentTab === 'diff'" class="tool-section">
      <div class="json-inputs">
        <textarea ref="leftTextarea" v-model="jsonInput" placeholder="Enter JSON (Left)..." rows="10"></textarea>
        <textarea ref="rightTextarea" v-model="jsonRightInput" placeholder="Enter JSON (Right)..." rows="10"></textarea>
      </div>
      <div class="row query-row">
        <input v-model="jsonQuery" placeholder="json path filter" 
          @input="showSuggestions = true"
          @blur="setTimeout(() => showSuggestions = false, 200)"
          @keydown="handleKeyDown" />
        <div class="diff-actions">
          <select v-model="diffTool" v-if="availableDiffTools.length > 0">
            <option v-for="tool in availableDiffTools" :key="tool" :value="tool">
              {{ tool }}
            </option>
          </select>
          <button @click="diffJson" :disabled="availableDiffTools.length === 0">
            {{ availableDiffTools.length > 0 ? 'Diff' : 'No Diff Tool' }}
          </button>
        </div>
        <button v-if="jsonOutput && !jsonOutput.startsWith('Error: ')" @click="saveJsonToFile">Save</button>
        <div v-if="showSuggestions && jsonKeys.length > 0" class="suggestions-dropdown">
          <div v-for="(key, index) in jsonKeys" :key="key" 
            class="suggestion-item" 
            :class="{ active: index === selectedIndex }"
            @mousedown.prevent="appendToQuery(key)">
            {{ key }}
          </div>
        </div>
      </div>
      <div class="json-outputs">
        <pre v-if="jsonOutput" class="output">{{ jsonOutput }}</pre>
        <pre v-if="jsonRightOutput" class="output">{{ jsonRightOutput }}</pre>
      </div>
    </section>

    <section v-if="currentTab === 'time'" class="tool-section">
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
  </main>
</template>

<style scoped>
.container {
  width: 100%;
  box-sizing: border-box;
  margin: 0;
  padding: 20px;
}

.tabs {
  display: flex;
  gap: 10px;
  margin-bottom: 20px;
  border-bottom: 1px solid #ccc;
  padding-bottom: 10px;
}

.tabs button {
  background: none;
  color: #666;
  border: 1px solid transparent;
  padding: 8px 16px;
}

.tabs button.active {
  background-color: #396cd8;
  color: white;
  border-radius: 4px;
}

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

.json-inputs, .json-outputs {
  display: flex;
  gap: 10px;
  margin-bottom: 10px;
}

.json-outputs .output {
  flex: 1;
  margin-top: 0;
}

.unit-select {
  flex: 0 0 150px;
}

.diff-actions {
  display: flex;
  gap: 0;
}

.diff-actions select {
  border-top-right-radius: 0;
  border-bottom-right-radius: 0;
  border-right: none;
  flex: 0 1 auto;
}

.diff-actions button {
  border-top-left-radius: 0;
  border-bottom-left-radius: 0;
}

input, textarea, select {
  flex: 1;
  padding: 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background-color: white;
  color: black;
}

textarea {
  min-height: calc(1.5em * 5 + 16px); /* 5 rows + padding */
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

.query-row {
  position: relative;
}

.suggestions-dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  background-color: white;
  border: 1px solid #ddd;
  border-radius: 4px;
  max-height: 200px;
  overflow-y: auto;
  z-index: 1000;
  box-shadow: 0 4px 6px rgba(0,0,0,0.1);
}

.suggestion-item {
  padding: 8px 12px;
  cursor: pointer;
  color: #333;
  text-align: left;
}

.suggestion-item:hover, .suggestion-item.active {
  background-color: #f0f0f0;
}

@media (prefers-color-scheme: dark) {
  .tabs {
    border-color: #444;
  }
  .tabs button {
    color: #aaa;
  }
  .tabs button.active {
    color: white;
  }
  .output {
    background-color: #1e1e1e;
    color: #d4d4d4;
  }
  .tool-section {
    border-color: #444;
  }
  input, textarea, select {
    background-color: #2a2a2a;
    border-color: #444;
    color: #d4d4d4;
  }
  .suggestions-dropdown {
    background-color: #2a2a2a;
    border-color: #444;
  }
  .suggestion-item {
    color: #d4d4d4;
  }
  .suggestion-item:hover, .suggestion-item.active {
    background-color: #3e3e3e;
  }
}
</style>
