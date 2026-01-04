<script setup>
import { ref, onMounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { useDebounce } from "../composables/useDebounce";

const props = defineProps({
  initialJson: String,
  initialQuery: String
});

const emit = defineEmits(['update:json', 'update:query']);

const jsonInput = ref(props.initialJson || "");
const jsonOutput = ref("");
const jsonQuery = ref(props.initialQuery || "");
const jsonKeys = ref([]);
const selectedIndex = ref(-1);
const showSuggestions = ref(false);

const { debounce } = useDebounce();

const lastSuccessfulOutput = ref("");

async function formatJson(input, outputRef) {
  try {
    outputRef.value = await invoke("format_json", { json: input });
  } catch (e) {
    outputRef.value = "Error: " + e;
  }
}

async function queryJson() {
  if (!jsonInput.value) {
    jsonOutput.value = "";
    jsonKeys.value = [];
    lastSuccessfulOutput.value = "";
    return;
  }
  
  if (jsonQuery.value && !jsonQuery.value.startsWith('$')) {
    jsonQuery.value = '$' + (jsonQuery.value.startsWith('.') ? '' : '.') + jsonQuery.value;
  }

  const query = (!jsonQuery.value || jsonQuery.value === '$') ? null : jsonQuery.value;

  if (!query) {
    await formatJson(jsonInput.value, jsonOutput);
    lastSuccessfulOutput.value = jsonOutput.value;
    await updateKeys();
    return;
  }

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

  await updateKeys();
}

async function updateKeys() {
  try {
    if (!jsonInput.value) {
      jsonKeys.value = [];
      return;
    }
    jsonKeys.value = await invoke("get_json_keys", { 
      json: jsonInput.value, 
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

watch([jsonInput, jsonQuery], debounce(() => {
  queryJson();
  emit('update:json', jsonInput.value);
  emit('update:query', jsonQuery.value);
}));

onMounted(() => {
  if (jsonInput.value) {
    queryJson();
  }
});
</script>

<template>
  <section class="tool-section">
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

.json-outputs .output {
  flex: 1;
  margin-top: 0;
}

input, textarea {
  flex: 1;
  padding: 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background-color: white;
  color: black;
}

textarea {
  min-height: calc(1.5em * 5 + 16px);
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
  input, textarea {
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
