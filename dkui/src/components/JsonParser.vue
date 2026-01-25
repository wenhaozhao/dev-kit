<script setup>
import {computed, nextTick, onMounted, onUnmounted, ref, watch} from "vue";
import {invoke} from "@tauri-apps/api/core";
import {listen} from "@tauri-apps/api/event";
import {open, save} from "@tauri-apps/plugin-dialog";
import {useDebounce} from "../composables/useDebounce";
import VueJsonPretty from 'vue-json-pretty';
import 'vue-json-pretty/lib/styles.css';

const _ = defineProps({
});

const emit = defineEmits(['update:json', 'update:query']);

const tabs = ref([]);
const activeTabIndex = ref(0);
const activeTab = computed(() => {
  return tabs.value[activeTabIndex.value] ?? null;
});
const tabsScrollContainer = ref(null);
invoke_jsonparser_init_tabs();
async function invoke_jsonparser_init_tabs() {
  try {
    const resp_tabs = await invoke("jsonparser_init_tabs");
    for (const resp of resp_tabs) {
      tabs.value.push({
        id: resp.id,
        idx: resp.idx,
        jsonInput: resp.json_input || "",
        jsonOutput: resp.json_output || "",
        jsonQuery: resp.json_query || "",
        jsonQuerying: false,
        jsonKeys: [],
        showSuggestions: false,
        selectedIndex: resp.selected_index,
      });
    }
    activeTabIndex.value = 0;
    ensureActiveTabVisible();
  } catch (e) {
    console.error(e);
  } finally {
  }
}

async function invoke_jsonparser_add_tab() {
  try {
    const resp = await invoke("jsonparser_add_tab");
    const new_tab = {
      id: resp.id,
      idx: resp.idx,
      jsonInput: "",
      jsonOutput: "",
      jsonQuery: "",
      jsonQuerying: false,
      jsonKeys: [],
      showSuggestions: false,
      selectedIndex: resp.selected_index,
    };
    tabs.value.push(new_tab);
    activeTabIndex.value = tabs.value.length - 1;
    ensureActiveTabVisible();
  } catch (e) {
    console.error(e);
  } finally {
  }
}

async function invoke_jsonparser_remove_tab(dst, indexOfTabs){
  try{
    await invoke("jsonparser_remove_tab", {tabId: dst.id});
    tabs.value.splice(indexOfTabs, 1);
    if (tabs.value.length === 0 ) {
      invoke_jsonparser_add_tab();
      return;
    }
    if (activeTabIndex.value >= tabs.value.length) {
      activeTabIndex.value = tabs.value.length - 1;
    }
    ensureActiveTabVisible();
  }catch (e){
    console.error(e);
  }finally {

  }
}

function addTab() {
  invoke_jsonparser_add_tab();
}

function removeTab(index) {
  const dst = tabs.value[index]
  invoke_jsonparser_remove_tab(dst, index);
}

const isDragging = ref(false);
const queryContainer = ref(null);

const { debounce } = useDebounce();

const parsedJsonOutput = computed(() => {
  if (!activeTab.value.jsonOutput || activeTab.value.jsonOutput.startsWith("Error: ")) {
    return null;
  }
  try {
    return JSON.parse(activeTab.value.jsonOutput);
  } catch (e) {
    return activeTab.value.jsonOutput;
  }
});

function scrollTabs(direction) {
  if (!tabsScrollContainer.value) return;
  const scrollAmount = 200;
  tabsScrollContainer.value.scrollBy({
    left: direction === 'left' ? -scrollAmount : scrollAmount,
    behavior: 'smooth'
  });
}

const showScrollButtons = ref(false);

function checkScroll() {
  if (!tabsScrollContainer.value) return;
  const { scrollWidth, clientWidth } = tabsScrollContainer.value;
  showScrollButtons.value = scrollWidth > clientWidth;
}

onMounted(() => {
  checkScroll();
  window.addEventListener('resize', checkScroll);
});

onUnmounted(() => {
  window.removeEventListener('resize', checkScroll);
});

watch(tabs, () => {
  nextTick(() => {
    checkScroll();
  });
}, { deep: true });

function ensureActiveTabVisible() {
  setTimeout(() => {
    if (!tabsScrollContainer.value) return;
    const activeTabEl = tabsScrollContainer.value.querySelector('.tab-item.active');
    if (activeTabEl) {
      activeTabEl.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'nearest' });
    }
  }, 50);
}

watch(activeTabIndex, () => {
  ensureActiveTabVisible();
});

async function processFile(path) {
  try {
    activeTab.value.jsonInput = path;
  } catch (e) {
    activeTab.value.jsonOutput = "Error reading file: " + e;
  }
}

async function openFile() {
  const selected = await open({
    multiple: false,
    directory: false,
  });
  if (selected) {
    await processFile(selected);
  }
}

async function queryJson(reload = false) {
  const currentTab = activeTab.value;
  if (!currentTab.jsonInput) {
    currentTab.jsonOutput = "";
    currentTab.jsonKeys = [];
    return;
  }
  currentTab.jsonQuerying = true;
  try {
    currentTab.jsonOutput = await invoke(
        "jsonparser_query_json",
        {
          json: currentTab.jsonInput,
          query: currentTab.jsonQuery,
          reload,
          tabId: currentTab.id,
        }
    );
  } catch (e) {
    currentTab.jsonOutput = "Error: " + e;
  } finally {
    currentTab.jsonQuerying = false
  }
  await updateKeys();
}

async function updateKeys() {
  const currentTab = activeTab.value;
  try {
    if (!currentTab.jsonInput) {
      currentTab.jsonKeys = [];
      return;
    }
    const paths = await invoke("jsonparser_search_json_paths", {
      tabId: currentTab.id,
      query: currentTab.jsonQuery || null
    });
    if (paths.length > 0) {
      currentTab.jsonKeys = paths
    }
  } catch (e) {
    console.error("Failed to fetch keys:", e);
  }
}

function appendToQuery(key) {
  const currentTab = activeTab.value;
  let val = currentTab.jsonQuery || "";
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
  currentTab.jsonQuery = val;
  currentTab.showSuggestions = false;
  currentTab.selectedIndex = -1;
}

function handleKeyDown(e) {
  const currentTab = activeTab.value;
  if (!currentTab.showSuggestions || currentTab.jsonKeys.length === 0) return;

  if (e.key === 'ArrowDown') {
    e.preventDefault();
    currentTab.selectedIndex = (currentTab.selectedIndex + 1) % currentTab.jsonKeys.length;
  } else if (e.key === 'ArrowUp') {
    e.preventDefault();
    currentTab.selectedIndex = (currentTab.selectedIndex - 1 + currentTab.jsonKeys.length) % currentTab.jsonKeys.length;
  } else if (e.key === 'Enter') {
    if (currentTab.selectedIndex >= 0) {
      e.preventDefault();
      appendToQuery(currentTab.jsonKeys[currentTab.selectedIndex].path);
    }
  } else if (e.key === 'Escape') {
    currentTab.showSuggestions = false;
    currentTab.selectedIndex = -1;
  }
}

async function saveJsonToFile() {
  const currentTab = activeTab.value;
  if (!currentTab.jsonOutput || currentTab.jsonOutput.startsWith("Error: ")) {
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
      await invoke("save_to_file", { path, content: currentTab.jsonOutput });
    }
  } catch (e) {
    currentTab.jsonOutput = "Error: " + e;
  }
}

watch(() => activeTab.value?.jsonKeys, () => {
  if (!activeTab.value) {
    return;
  }
  activeTab.value.selectedIndex = -1;
});

watch([() => activeTab.value?.jsonInput, () => activeTab.value?.jsonQuery, () => activeTabIndex.value], debounce(() => {
  if (!activeTab.value) {
    return;
  }
  queryJson();
  emit('update:json', activeTab.value.jsonInput);
  emit('update:query', activeTab.value.jsonQuery);
}, 500));

onMounted(async () => {
  await listen("tauri://drag-drop", (event) => {
    isDragging.value = false;
    const paths = event.payload.paths;
    if (paths && paths.length > 0) {
      processFile(paths[0]);
    }
  });

  await listen("tauri://drag-over", () => {
    isDragging.value = true;
  });

  await listen("tauri://drag-leave", () => {
    isDragging.value = false;
  });

  const handleClickOutside = (e) => {
    if (queryContainer.value && !queryContainer.value.contains(e.target)) {
      activeTab.value.showSuggestions = false;
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
    <div class="tabs-header">
      <button v-if="showScrollButtons" class="scroll-button" @click="scrollTabs('left')" title="Scroll Left">
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="15 18 9 12 15 6"></polyline>
        </svg>
      </button>
      <div class="tabs-container" ref="tabsScrollContainer">
        <div v-for="(tab, index) in tabs" :key="index" 
          class="tab-item" 
          :class="{ active: index === activeTabIndex }"
          @click="activeTabIndex = index">
          <span class="tab-title">JSON {{ tab.idx }}</span>
          <button class="tab-close" @click.stop="removeTab(index)" v-if="tabs.length > 1">
            <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </button>
        </div>
      </div>
      <button v-if="showScrollButtons" class="scroll-button" @click="scrollTabs('right')" title="Scroll Right">
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="9 18 15 12 9 6"></polyline>
        </svg>
      </button>
      <button class="add-tab-button" @click="addTab" title="Add Tab">
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="12" y1="5" x2="12" y2="19"></line>
          <line x1="5" y1="12" x2="19" y2="12"></line>
        </svg>
      </button>
    </div>
    <div class="json-inputs">
      <div v-if="activeTab" class="textarea-container" :class="{ dragging: isDragging }">
        <textarea v-model="activeTab.jsonInput" placeholder="Enter JSON..." rows="5"></textarea>
        <div class="textarea-actions">
          <button v-if="activeTab.jsonInput" class="action-button" @click="queryJson(true)" title="Run" :disabled="activeTab.jsonQuerying">
            <svg v-if="!activeTab.jsonQuerying" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polygon points="5 3 19 12 5 21 5 3"></polygon>
            </svg>
            <svg v-else class="spinner" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 12a9 9 0 1 1-6.219-8.56"></path>
            </svg>
          </button>
          <button class="action-button" @click="openFile" title="Open File">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"></path>
              <polyline points="13 2 13 9 20 9"></polyline>
            </svg>
          </button>
          <button v-if="activeTab.jsonInput" class="action-button" @click="activeTab.jsonInput = ''" title="Clear">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </button>
        </div>
      </div>
    </div>
    <div v-if="activeTab" class="row query-row" ref="queryContainer">
      <input v-model="activeTab.jsonQuery" placeholder="json path/key/val filter"
        @input="activeTab.showSuggestions = true"
        @focus="activeTab.showSuggestions = true"
        @keydown="handleKeyDown" />
      <div v-if="activeTab.showSuggestions && activeTab.jsonKeys.length > 0" class="suggestions-dropdown">
        <div v-for="(key, index) in activeTab.jsonKeys" :key="key.path"
          class="suggestion-item" 
          :class="{ active: index === activeTab.selectedIndex }"
          @mousedown.prevent="appendToQuery(key.path)">
          {{ key.path }} {{ !!key.val ? ` -> ${key.val}` : '' }}
        </div>
      </div>
    </div>
    <div v-if="activeTab" class="json-outputs">
      <div v-if="activeTab.jsonOutput" class="output">
        <div class="output-actions">
          <button v-if="!activeTab.jsonOutput.startsWith('Error: ')" class="action-button" @click="saveJsonToFile" title="Save to File">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"></path>
              <polyline points="17 21 17 13 7 13 7 21"></polyline>
              <polyline points="7 3 7 8 15 8"></polyline>
            </svg>
          </button>
        </div>
        <div v-if="activeTab.jsonOutput.startsWith('Error: ')" class="error-msg">{{ activeTab.jsonOutput }}</div>
        <vue-json-pretty
          v-else
          :data="parsedJsonOutput"
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

.tabs-header {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-bottom: 16px;
  border-bottom: 1px solid #eee;
  padding-bottom: 0px;
}

.tabs-container {
  display: flex;
  align-items: flex-end;
  gap: 4px;
  overflow-x: auto;
  scrollbar-width: none; /* Firefox */
  -ms-overflow-style: none;  /* IE and Edge */
  flex: 1;
  scroll-behavior: smooth;
}

.tabs-container::-webkit-scrollbar {
  display: none; /* Chrome, Safari and Opera */
}

.scroll-button {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4px;
  background: transparent;
  color: #999;
  border: none;
  cursor: pointer;
  transition: color 0.2s;
}

.scroll-button:hover {
  color: #396cd8;
}

.tab-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  background: #f5f5f5;
  border-radius: 4px 4px 0 0;
  cursor: pointer;
  font-size: 13px;
  color: #666;
  border: 1px solid transparent;
  border-bottom: none;
  transition: all 0.2s;
  white-space: nowrap;
  flex-shrink: 0;
}

.tab-item:hover {
  background: #eee;
}

.tab-item.active {
  background: white;
  color: #396cd8;
  border-color: #eee;
  padding-bottom: 8px;
  font-weight: 500;
  position: relative;
  z-index: 1;
}

/* Add a line to cover the bottom border for the active tab */
.tab-item.active::after {
  content: '';
  position: absolute;
  bottom: -1px;
  left: 0;
  right: 0;
  height: 1px;
  background: white;
}

.tab-close {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2px;
  border: none;
  background: transparent;
  color: #999;
  border-radius: 50%;
  cursor: pointer;
}

.tab-close:hover {
  background: rgba(0, 0, 0, 0.1);
  color: #666;
}

.add-tab-button {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4px;
  background: transparent;
  color: #999;
  border: 1px dashed #ccc;
  border-radius: 4px;
  cursor: pointer;
  margin-left: 4px;
}

.add-tab-button:hover {
  color: #396cd8;
  border-color: #396cd8;
  background: rgba(57, 108, 216, 0.05);
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

input, textarea {
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

.action-button:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

.spinner {
  animation: rotate 2s linear infinite;
}

@keyframes rotate {
  100% {
    transform: rotate(360deg);
  }
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

:root.dark-mode .output {
  background-color: #1e1e1e;
  color: #d4d4d4;
}
:root.dark-mode :deep(.vjs-tree-node:hover) {
  background-color: #3e3e3e;
}
:root.dark-mode :deep(.vjs-value__string) {
  color: #ce9178;
}
:root.dark-mode :deep(.vjs-value__number) {
  color: #b5cea8;
}
:root.dark-mode :deep(.vjs-key) {
  color: #9cdcfe;
}
:root.dark-mode .tool-section {
  border-color: #444;
}
:root.dark-mode input, :root.dark-mode textarea {
  background-color: #2a2a2a;
  border-color: #444;
  color: #d4d4d4;
}
:root.dark-mode .suggestions-dropdown {
  background-color: #2a2a2a;
  border-color: #444;
}
:root.dark-mode .suggestion-item {
  color: #d4d4d4;
}
:root.dark-mode .suggestion-item:hover, :root.dark-mode .suggestion-item.active {
  background-color: #3e3e3e;
}
:root.dark-mode .action-button {
  background: rgba(255, 255, 255, 0.05);
  color: #aaa;
}
:root.dark-mode .action-button:hover {
  background: rgba(255, 255, 255, 0.15);
  color: #fff;
}
:root.dark-mode .textarea-container.dragging textarea {
  background-color: rgba(0, 122, 255, 0.1);
}
:root.dark-mode .tabs-header {
  border-bottom-color: #444;
}
:root.dark-mode .tab-item {
  background: #252525;
  color: #aaa;
}
:root.dark-mode .tab-item:hover {
  background: #333;
}
:root.dark-mode .tab-item.active {
  background: #2a2a2a;
  color: #396cd8;
  border-color: #444;
}
:root.dark-mode .tab-close:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #ccc;
}
:root.dark-mode .add-tab-button {
  border-color: #444;
}
</style>
