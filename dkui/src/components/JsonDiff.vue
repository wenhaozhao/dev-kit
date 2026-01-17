<script setup>
import { ref, onMounted, onUnmounted, watch, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { save, open } from "@tauri-apps/plugin-dialog";
import { useDebounce } from "../composables/useDebounce";
import VueJsonPretty from 'vue-json-pretty';
import 'vue-json-pretty/lib/styles.css';

const props = defineProps({
  initialLeftJson: String,
  initialRightJson: String,
  initialQuery: String
});

const emit = defineEmits(['update:leftJson', 'update:rightJson', 'update:query']);

const jsonLeftInput = ref(props.initialLeftJson || "");
const jsonRightInput = ref(props.initialRightJson || "");
const jsonOutput = ref("");
const jsonRightOutput = ref("");
const jsonQuery = ref(props.initialQuery || "");
const diffTool = ref("");
const availableDiffTools = ref([]);
const jsonKeys = ref([]);
const selectedIndex = ref(-1);
const showSuggestions = ref(false);
const isDraggingLeft = ref(false);
const isDraggingRight = ref(false);
const queryContainer = ref(null);

const leftTextarea = ref(null);
const rightTextarea = ref(null);
let isSyncing = false;

const { debounce } = useDebounce();

const lastSuccessfulOutput = ref("");
const lastSuccessfulRightOutput = ref("");

const parsedJsonOutput = computed(() => {
  if (!jsonOutput.value || jsonOutput.value.startsWith("Error: ")) {
    return null;
  }
  try {
    return JSON.parse(jsonOutput.value);
  } catch (e) {
    return jsonOutput.value;
  }
});

const parsedJsonRightOutput = computed(() => {
  if (!jsonRightOutput.value || jsonRightOutput.value.startsWith("Error: ")) {
    return null;
  }
  try {
    return JSON.parse(jsonRightOutput.value);
  } catch (e) {
    return jsonRightOutput.value;
  }
});

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

  let observer = new ResizeObserver(syncHeight);
  if (leftTextarea.value) observer.observe(leftTextarea.value);
  if (rightTextarea.value) observer.observe(rightTextarea.value);

  if (jsonLeftInput.value || jsonRightInput.value) {
    queryJson();
  }

  await listen("tauri://drag-drop", (event) => {
    const paths = event.payload.paths;
    if (paths && paths.length > 0) {
      // Logic for determining which side to drop into:
      // Since it's a split screen, we might need to check mouse position or target element.
      // However, the standard `tauri://drag-drop` doesn't easily give target element.
      // For now, let's use the `isDraggingLeft/Right` states set by `tauri://drag-over`.
      if (isDraggingLeft.value) {
        processFile(paths[0], jsonLeftInput);
      } else if (isDraggingRight.value) {
        processFile(paths[0], jsonRightInput);
      }
    }
  });

  await listen("tauri://drag-over", (event) => {
    // Determine which side based on x position
    if (event.payload && event.payload.position) {
      const {x} = event.payload.position;
      const windowWidth = window.innerWidth;
      const midpoint = windowWidth / 2;

      if (x < midpoint) {
        isDraggingLeft.value = true;
        isDraggingRight.value = false;
      } else {
        isDraggingLeft.value = false;
        isDraggingRight.value = true;
      }
    }
  });

  await listen("tauri://drag-leave", () => {
    isDraggingLeft.value = false;
    isDraggingRight.value = false;
  });

  const handleClickOutside = (e) => {
    if (queryContainer.value && !queryContainer.value.contains(e.target)) {
      showSuggestions.value = false;
    }
  };

  document.addEventListener('click', handleClickOutside);

  onUnmounted(() => {
    document.removeEventListener('click', handleClickOutside);
  });
});

async function processFile(path, inputRef) {
  try {
    inputRef.value = path;
  } catch (e) {
    console.error("Error reading file:", e);
  }
}

async function openLeftFile() {
  const selected = await open({
    multiple: false,
    directory: false,
  });
  if (selected) {
    await processFile(selected, jsonLeftInput);
  }
}

async function openRightFile() {
  const selected = await open({
    multiple: false,
    directory: false,
  });
  if (selected) {
    await processFile(selected, jsonRightInput);
  }
}

async function queryJson() {
  if (!jsonLeftInput.value && !jsonRightInput.value) {
    jsonOutput.value = "";
    jsonRightOutput.value = "";
    jsonKeys.value = [];
    lastSuccessfulOutput.value = "";
    lastSuccessfulRightOutput.value = "";
    return;
  }

  if (jsonLeftInput.value) {
    try {
      jsonOutput.value = await invoke("query_json", { json: jsonLeftInput.value, query: jsonQuery.value });
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
      jsonRightOutput.value = await invoke("query_json", { json: jsonRightInput.value, query: jsonQuery.value });
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
    const jsonToProcess = jsonLeftInput.value || jsonRightInput.value;
    if (!jsonToProcess) {
      jsonKeys.value = [];
      return;
    }
    const paths = await invoke("search_json_paths", {
      json: jsonToProcess,
      query: jsonQuery.value || null
    });
    if (paths.length > 0) {
      jsonKeys.value = paths
    }
  } catch (e) {
    console.error("Failed to fetch keys:", e);
  }
}

function appendToQuery(key) {
  let val = jsonQuery.value || "";
  if (key.startsWith("$")) {
    val = key;
  }else{
    if (val.startsWith("$")) {
      if (val.endsWith(".")) {
        val += key;
      } else {
        val += "." + key;
      }
    } else {
      val = key;
    }
  }
  jsonQuery.value = val;
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
      left: jsonLeftInput.value,
      right: jsonRightInput.value, 
      query: jsonQuery.value || null,
      diffTool: diffTool.value 
    });
  } catch (e) {
    jsonOutput.value = "Error: " + e;
  }
}

async function saveJsonToFile(content) {
  if (!content || content.startsWith("Error: ")) {
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
      await invoke("save_to_file", { path, content });
    }
  } catch (e) {
    console.error("Save failed:", e);
  }
}

watch(jsonKeys, () => {
  selectedIndex.value = -1;
});

watch([jsonLeftInput, jsonRightInput, jsonQuery], debounce(() => {
  queryJson();
  emit('update:leftJson', jsonLeftInput.value);
  emit('update:rightJson', jsonRightInput.value);
  emit('update:query', jsonQuery.value);
}));
</script>

<template>
  <section class="tool-section">
    <div class="json-inputs">
      <div class="textarea-container" :class="{ dragging: isDraggingLeft }"
        @dragover="isDraggingLeft = true"
        @dragleave="isDraggingLeft = false"
        @drop="isDraggingLeft = false">
        <textarea id="leftTextarea" ref="leftTextarea" v-model="jsonLeftInput" placeholder="Enter JSON (Left)..." rows="5"></textarea>
        <div class="textarea-actions">
          <button class="action-button" @click="openLeftFile" title="Open File">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"></path>
              <polyline points="13 2 13 9 20 9"></polyline>
            </svg>
          </button>
          <button v-if="jsonLeftInput" class="action-button" @click="jsonLeftInput = ''" title="Clear">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </button>
        </div>
      </div>
      <div class="textarea-container" :class="{ dragging: isDraggingRight }"
        @dragover="isDraggingRight = true"
        @dragleave="isDraggingRight = false"
        @drop="isDraggingRight = false">
        <textarea id="rightTextarea" ref="rightTextarea" v-model="jsonRightInput" placeholder="Enter JSON (Right)..." rows="5"></textarea>
        <div class="textarea-actions">
          <button class="action-button" @click="openRightFile" title="Open File">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"></path>
              <polyline points="13 2 13 9 20 9"></polyline>
            </svg>
          </button>
          <button v-if="jsonRightInput" class="action-button" @click="jsonRightInput = ''" title="Clear">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </button>
        </div>
      </div>
    </div>
    <div class="row query-row" ref="queryContainer">
      <input v-model="jsonQuery" placeholder="json path filter" 
        @input="showSuggestions = true"
        @focus="showSuggestions = true"
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
      <div v-if="jsonOutput" class="output">
        <div class="output-actions">
          <button v-if="!jsonOutput.startsWith('Error: ')" class="action-button" @click="saveJsonToFile(jsonOutput)" title="Save to File">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"></path>
              <polyline points="17 21 17 13 7 13 7 21"></polyline>
              <polyline points="7 3 7 8 15 8"></polyline>
            </svg>
          </button>
        </div>
        <div v-if="jsonOutput.startsWith('Error: ')" class="error-msg">{{ jsonOutput }}</div>
        <vue-json-pretty
          v-else
          :data="parsedJsonOutput"
          :show-length="true"
          :deep="3"
        />
      </div>
      <div v-if="jsonRightOutput" class="output">
        <div class="output-actions">
          <button v-if="!jsonRightOutput.startsWith('Error: ')" class="action-button" @click="saveJsonToFile(jsonRightOutput)" title="Save to File">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"></path>
              <polyline points="17 21 17 13 7 13 7 21"></polyline>
              <polyline points="7 3 7 8 15 8"></polyline>
            </svg>
          </button>
        </div>
        <div v-if="jsonRightOutput.startsWith('Error: ')" class="error-msg">{{ jsonRightOutput }}</div>
        <vue-json-pretty
          v-else
          :data="parsedJsonRightOutput"
          :show-length="true"
          :deep="3"
        />
      </div>
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

.textarea-container.dragging textarea {
  border-color: #007aff;
  background-color: rgba(0, 122, 255, 0.05);
}

.textarea-actions {
  position: absolute;
  top: 8px;
  right: 8px;
  display: flex;
  gap: 4px;
  z-index: 10;
}

.action-button {
  padding: 4px;
  background: rgba(0, 0, 0, 0.05);
  color: #666;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.2s, color 0.2s;
}

.action-button:hover {
  background: rgba(0, 0, 0, 0.15);
  color: #333;
}

.json-outputs .output {
  position: relative;
  flex: 1;
  margin-top: 0;
  width: 0;
}

.output-actions {
  position: absolute;
  top: 8px;
  right: 8px;
  display: flex;
  gap: 4px;
  z-index: 10;
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
  overflow: auto;
}

.error-msg {
  color: #f44336;
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
  :deep(.vjs-tree-node:hover) {
    background-color: #3e3e3e;
  }
  :deep(.vjs-value__string) {
    color: #ce9178;
  }
  :deep(.vjs-value__number) {
    color: #b5cea8;
  }
  :deep(.vjs-key) {
    color: #9cdcfe;
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
  .action-button {
    background: rgba(255, 255, 255, 0.05);
    color: #aaa;
  }
  .action-button:hover {
    background: rgba(255, 255, 255, 0.15);
    color: #fff;
  }
  .textarea-container.dragging textarea {
    background-color: rgba(0, 122, 255, 0.1);
  }
}
</style>
