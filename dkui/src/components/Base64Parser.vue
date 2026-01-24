<script setup>
import { ref, watch, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";

const input = ref("");
const output = ref("");
const mode = ref("decode"); // decode, encode
const urlSafe = ref(false);
const noPad = ref(false);
const isDragging = ref(false);

let debounceTimer = null;
function debounce(fn, delay = 300) {
  return (...args) => {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => fn(...args), delay);
  };
}

async function processFile(path) {
  try {
    if (mode.value === 'encode') {
      // 编码模式下，直接读取文件并转为 base64
      input.value = path;
    } else {
      // 解码模式下，读取文件内容作为输入
      input.value = path;
    }
  } catch (e) {
    output.value = "Error reading file: " + e;
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
});

async function handleBase64() {
  if (!input.value || input.value.startsWith("File: ")) {
    return;
  }

  try {
    if (mode.value === "decode") {
      output.value = await invoke("base64_decode", { 
        input: input.value,
        urlSafe: urlSafe.value,
        noPad: noPad.value
      });
    } else if (mode.value === "encode") {
      output.value = await invoke("base64_encode", { 
        input: input.value,
        urlSafe: urlSafe.value,
        noPad: noPad.value
      });
    }
  } catch (e) {
    output.value = "Error: " + e;
  }
}

async function onClickDecode() {
  mode.value = 'decode'
  if (output.value && !output.value.startsWith("Error:")) {
    input.value = output.value
  }
}

async function onClickEncode() {
  mode.value = 'encode'
  if (output.value && !output.value.startsWith("Error:")) {
    input.value = output.value
  }
}

async function copyToClipboard(e) {
  const text = e.target.innerText;
  try {
    await navigator.clipboard.writeText(text);
    const originalBg = e.target.style.backgroundColor;
    e.target.style.backgroundColor = '#d4edda';
    setTimeout(() => {
      e.target.style.backgroundColor = originalBg;
    }, 500);
  } catch (err) {
    console.error('Failed to copy: ', err);
  }
}

watch([input, mode, urlSafe, noPad], debounce(() => {
  handleBase64();
}));
</script>

<template>
  <section class="tool-section">
    <div class="row">
      <div class="textarea-container" :class="{ dragging: isDragging }">
        <textarea v-model="input" :placeholder="`Enter text or drop file to ${mode}...`" rows="5"></textarea>
        <div class="textarea-actions">
          <button class="action-button" @click="openFile" title="Open File">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"></path>
              <polyline points="13 2 13 9 20 9"></polyline>
            </svg>
          </button>
          <button v-if="input" class="action-button" @click="input = ''; output = ''" title="Clear">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </button>
        </div>
      </div>
    </div>
    
    <div class="button-row">
      <div class="checkbox-group">
        <label>
          <input type="checkbox" v-model="urlSafe"> URL Safe
        </label>
        <label>
          <input type="checkbox" v-model="noPad"> No Padding
        </label>
      </div>

      <div class="button-group">
        <button 
          :class="{ active: mode === 'decode' }" 
          @click="onClickDecode"
        >
          Decode
        </button>
        <button 
          :class="{ active: mode === 'encode' }" 
          @click="onClickEncode"
        >
          Encode
        </button>
      </div>
    </div>
    
    <div v-if="output" class="output-container">
      <div class="copy-tip">Double Click to Copy</div>
      <pre class="output" @dblclick="copyToClipboard">{{ output }}</pre>
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

textarea {
  flex: 1;
  padding: 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background-color: white;
  color: black;
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

.button-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.button-group {
  display: flex;
  flex-direction: row;
  gap: 0;
  border: 1px solid #ddd;
  border-radius: 4px;
  overflow: hidden;
}

.button-group button {
  padding: 8px 12px;
  border: none;
  background-color: white;
  color: black;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.2s;
  font-weight: normal;
  border-radius: 0;
}

.button-group button:not(:last-child) {
  border-right: 1px solid #ddd;
}

.button-group button.active {
  background-color: #007aff;
  color: white;
}

button:hover:not(.active) {
  background-color: #f5f5f5;
}

.checkbox-group {
  display: flex;
  gap: 20px;
  color: #666;
  font-size: 14px;
}

.checkbox-group label {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
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
  width: 100%;
  box-sizing: border-box;
}

.output:hover {
  background-color: #e8e8e8;
}

:root.dark-mode .output {
  background-color: #1e1e1e;
  color: #d4d4d4;
}
:root.dark-mode .output:hover {
  background-color: #2a2a2a;
}
:root.dark-mode .tool-section {
  border-color: #444;
}
:root.dark-mode textarea, :root.dark-mode .button-group {
  border-color: #444;
}
:root.dark-mode textarea {
  background-color: #2a2a2a;
  color: #d4d4d4;
}
:root.dark-mode button {
  background-color: #2a2a2a;
  color: #d4d4d4;
}
:root.dark-mode button:not(:last-child) {
  border-right-color: #444;
}
:root.dark-mode button.active {
  background-color: #007aff;
  color: white;
}
:root.dark-mode button:hover:not(.active) {
  background-color: #3a3a3a;
}
:root.dark-mode .checkbox-group {
  color: #aaa;
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
</style>
