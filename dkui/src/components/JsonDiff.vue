<script setup>
import { ref, onMounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { useDebounce } from "../composables/useDebounce";

const props = defineProps({
  initialLeftJson: String,
  initialRightJson: String,
  initialQuery: String
});

const emit = defineEmits(['update:leftJson', 'update:rightJson', 'update:query']);

const jsonInput = ref(props.initialLeftJson || "");
const jsonRightInput = ref(props.initialRightJson || "");
const jsonOutput = ref("");
const jsonRightOutput = ref("");
const jsonQuery = ref(props.initialQuery || "");
const diffTool = ref("");
const availableDiffTools = ref([]);
const jsonKeys = ref([]);
const selectedIndex = ref(-1);
const showSuggestions = ref(false);

const leftTextarea = ref(null);
const rightTextarea = ref(null);
let isSyncing = false;

const { debounce } = useDebounce();

const lastSuccessfulOutput = ref("");
const lastSuccessfulRightOutput = ref("");

onMounted(async () => {
  try {
    availableDiffTools.value = await invoke("get_available_diff_tools");
    if (availableDiffTools.value.length > 0) {
      diffTool.value = availableDiffTools.value[0];
    }
  } catch (e) {
    console.error("Failed to fetch diff tools:", e);
  }

  // Sync textarea heights
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

  if (!observer) {
    observer = new ResizeObserver(syncHeight);
  }
  if (leftTextarea.value) observer.observe(leftTextarea.value);
  if (rightTextarea.value) observer.observe(rightTextarea.value);

  if (jsonInput.value || jsonRightInput.value) {
    queryJson();
  }
});

async function formatJson(input, outputRef) {
  try {
    outputRef.value = await invoke("format_json", { json: input });
  } catch (e) {
    outputRef.value = "Error: " + e;
  }
}

async function queryJson() {
  if (!jsonInput.value && !jsonRightInput.value) {
    jsonOutput.value = "";
    jsonRightOutput.value = "";
    jsonKeys.value = [];
    lastSuccessfulOutput.value = "";
    lastSuccessfulRightOutput.value = "";
    return;
  }
  
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

watch(jsonKeys, () => {
  selectedIndex.value = -1;
});

watch([jsonInput, jsonRightInput, jsonQuery], debounce(() => {
  queryJson();
  emit('update:leftJson', jsonInput.value);
  emit('update:rightJson', jsonRightInput.value);
  emit('update:query', jsonQuery.value);
}));
</script>

<template>
  <section class="tool-section">
    <div class="json-inputs">
      <div class="textarea-container">
        <textarea ref="leftTextarea" v-model="jsonInput" placeholder="Enter JSON (Left)..." rows="5"></textarea>
        <button v-if="jsonInput" class="clear-button" @click="jsonInput = ''" title="Clear">
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>
      <div class="textarea-container">
        <textarea ref="rightTextarea" v-model="jsonRightInput" placeholder="Enter JSON (Right)..." rows="5"></textarea>
        <button v-if="jsonRightInput" class="clear-button" @click="jsonRightInput = ''" title="Clear">
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>
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

.json-inputs, .json-outputs {
  display: flex;
  gap: 10px;
  margin-bottom: 10px;
}

.textarea-container {
  position: relative;
  flex: 1;
  display: flex;
  width: 0;
}

.clear-button {
  position: absolute;
  top: 8px;
  right: 8px;
  padding: 4px;
  background: rgba(0, 0, 0, 0.1);
  color: #666;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10;
  transition: background 0.2s, color 0.2s;
}

.clear-button:hover {
  background: rgba(0, 0, 0, 0.2);
  color: #333;
}

.json-outputs .output {
  flex: 1;
  margin-top: 0;
  width: 0;
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
  min-height: calc(1.2em * 5 + 16px);
  line-height: 1.2;
  resize: vertical;
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
  .clear-button {
    background: rgba(255, 255, 255, 0.1);
    color: #aaa;
  }
  .clear-button:hover {
    background: rgba(255, 255, 255, 0.2);
    color: #fff;
  }
  .suggestion-item {
    color: #d4d4d4;
  }
  .suggestion-item:hover, .suggestion-item.active {
    background-color: #3e3e3e;
  }
}
</style>
