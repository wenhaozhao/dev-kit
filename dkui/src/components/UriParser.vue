<script setup>
import {ref, watch, onMounted, onUnmounted} from "vue";
import {invoke} from "@tauri-apps/api/core";

const uriInput = ref("");
const uriFilter = ref("");
const parseResult = ref([]);
const showSuggestions = ref(false);
const selectedIndex = ref(-1);
const availableComponents = ref([]);
const filteredSuggestions = ref([]);
const queryContainer = ref(null);

function updateFilteredSuggestions() {
  const parts = uriFilter.value.split(",");
  const lastPart = parts[parts.length - 1].trim().toLowerCase();

  if (lastPart === "") {
    filteredSuggestions.value = availableComponents.value;
  } else {
    filteredSuggestions.value = availableComponents.value.filter(c =>
      c.startsWith(lastPart) && !parts.slice(0, -1).map(p => p.trim().toLowerCase()).includes(c)
    );
  }

  if (filteredSuggestions.value.length === 0) {
    showSuggestions.value = false;
  }
}

function selectSuggestion(suggestion) {
  const parts = uriFilter.value.split(",");
  parts[parts.length - 1] = suggestion;
  uriFilter.value = parts.join(", ") + ", ";
  showSuggestions.value = false;
  selectedIndex.value = -1;
}

function handleKeyDown(e) {
  if (!showSuggestions.value || filteredSuggestions.value.length === 0) return;

  if (e.key === 'ArrowDown') {
    e.preventDefault();
    selectedIndex.value = (selectedIndex.value + 1) % filteredSuggestions.value.length;
  } else if (e.key === 'ArrowUp') {
    e.preventDefault();
    selectedIndex.value = (selectedIndex.value - 1 + filteredSuggestions.value.length) % filteredSuggestions.value.length;
  } else if (e.key === 'Enter') {
    if (selectedIndex.value >= 0) {
      e.preventDefault();
      selectSuggestion(filteredSuggestions.value[selectedIndex.value]);
    }
  } else if (e.key === 'Escape') {
    showSuggestions.value = false;
    selectedIndex.value = -1;
  }
}

let debounceTimer = null;
function debounce(fn, delay = 300) {
  return (...args) => {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => fn(...args), delay);
  };
}

async function handleParse() {
  if (!uriInput.value) {
    parseResult.value = [];
    return;
  }

  try {
    const cleanUri = uriInput.value;//.trim().replace(/[\r\n]+/g, "");
    const filter = uriFilter.value
      ? uriFilter.value.split(",").map((s) => s.trim()).filter((s) => s)
      : null;
    parseResult.value = await invoke("parse_uri", { uri: cleanUri, filter });

    // Update available components based on current result
    if (parseResult.value && Array.isArray(parseResult.value)) {
      parseResult.value.forEach(item => {
        if (item.name && !availableComponents.value.includes(item.name)) {
          availableComponents.value.push(item.name);
        }
        // If it's query, extract keys from its map value
        if (item.name === "query" && item.value && typeof item.value === "object" && !Array.isArray(item.value)) {
          Object.keys(item.value).forEach(key => {
            if (!availableComponents.value.includes(key)) {
              availableComponents.value.push(key);
            }
          });
        }
      });
    }

  } catch (e) {
    parseResult.value = [{ name: "Error", value: e.toString() }];
  }
}

async function copyToClipboard(e) {
  const text = e.target.innerText;
  try {
    await invoke("copy_to_clipboard", { text });
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

watch([uriInput, uriFilter], debounce(() => {
  handleParse();
}));

onMounted(() => {
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
</script>

<template>
  <section class="tool-section">
    <div class="row">
      <div class="textarea-container">
        <textarea
          v-model="uriInput"
          placeholder="Enter URI to parse (supports multi-line)..."
          rows="5"
        ></textarea>
        <button v-if="uriInput" class="clear-button" @click="uriInput = ''" title="Clear">
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>
    </div>

    <div class="row filter-row" ref="queryContainer">
      <input
        v-model="uriFilter"
        placeholder="component filter: scheme, authority, host, port, path, query (comma separated)"
        @input="showSuggestions = true; updateFilteredSuggestions()"
        @focus="showSuggestions = true; updateFilteredSuggestions()"
        @keydown="handleKeyDown"
      />
      <div v-if="showSuggestions && filteredSuggestions.length > 0" class="suggestions-dropdown">
        <div v-for="(suggestion, index) in filteredSuggestions" :key="suggestion"
          class="suggestion-item"
          :class="{ active: index === selectedIndex }"
          @mousedown.prevent="selectSuggestion(suggestion)">
          {{ suggestion }}
        </div>
      </div>
    </div>

    <div v-if="parseResult.length > 0" class="parse-result">
      <div v-for="item in parseResult" :key="item.name" class="component-item">
        <span class="component-name">{{ item.name }}:</span>
        <div v-if="typeof item.value === 'object'" class="component-children">
              <div v-for="k in Object.keys(item.value)" class="component-item">
                <span class="component-name">{{ k }}:</span>
                <div class="output-container">
                  <div class="copy-tip">Double Click to Copy</div>
                  <pre class="component-value" @dblclick="copyToClipboard">{{ typeof item.value[k] === 'object' ? JSON.stringify(item.value[k], null, 2) : item.value[k] }}</pre>
                </div>
              </div>
        </div>
        <div v-else class="output-container">
          <div class="copy-tip">Double Click to Copy</div>
          <pre class="component-value" @dblclick="copyToClipboard">{{ typeof item.value === 'object' ? JSON.stringify(item.value, null, 2) : item.value }}</pre>
        </div>
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

textarea, input {
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

.parse-result {
  margin-top: 10px;
}

.component-item {
  margin-bottom: 8px;
  display: flex;
  align-items: flex-start;
  gap: 8px;
}

.component-children {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.component-name {
  font-weight: bold;
  color: #555;
  white-space: nowrap;
  min-width: 80px;
}

.output-container {
  position: relative;
  flex: 1;
}

.copy-tip {
  position: absolute;
  top: 2px;
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

.component-value {
  margin: 0;
  padding: 4px 8px;
  background-color: #f0f0f0;
  border-radius: 4px;
  font-family: monospace;
  white-space: pre-wrap;
  word-break: break-all;
  flex: 1;
  cursor: pointer;
  transition: background-color 0.2s;
}

.component-value:hover {
  background-color: #e8e8e8;
}

.filter-row {
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

:root.dark-mode .component-value {
  background-color: #1e1e1e;
  color: #d4d4d4;
}
:root.dark-mode .component-value:hover {
  background-color: #2a2a2a;
}
:root.dark-mode .tool-section {
  border-color: #444;
}
:root.dark-mode textarea {
  background-color: #2a2a2a;
  border-color: #444;
  color: #d4d4d4;
}
:root.dark-mode .component-name {
  color: #aaa;
}
:root.dark-mode .suggestions-dropdown {
  background-color: #2a2a2a;
  border-color: #444;
}
:root.dark-mode .clear-button {
  background: rgba(255, 255, 255, 0.1);
  color: #aaa;
}
:root.dark-mode .clear-button:hover {
  background: rgba(255, 255, 255, 0.2);
  color: #fff;
}
:root.dark-mode .suggestion-item {
  color: #d4d4d4;
}
:root.dark-mode .suggestion-item:hover, :root.dark-mode .suggestion-item.active {
  background-color: #3e3e3e;
}
</style>
