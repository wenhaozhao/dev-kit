<script setup>
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";

const uriInput = ref("");
const uriOutput = ref("");
const mode = ref("decode"); // decode, encode

let debounceTimer = null;
function debounce(fn, delay = 300) {
  return (...args) => {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => fn(...args), delay);
  };
}

async function handleUri() {
  if (!uriInput.value) {
    uriOutput.value = "";
    return;
  }

  try {
    if (mode.value === "decode") {
      uriOutput.value = await invoke("decode_uri", { uri: uriInput.value });
    } else if (mode.value === "encode") {
      uriOutput.value = await invoke("encode_uri", { uri: uriInput.value });
    }
  } catch (e) {
    uriOutput.value = "Error: " + e;
  }
}

async function onClickDecode() {
  mode.value = 'decode'
  uriInput.value = uriOutput.value
}

async function onClickEncode() {
  mode.value = 'encode'
  uriInput.value = uriOutput.value
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


watch([uriInput, mode], debounce(() => {
  handleUri();
}));
</script>

<template>
  <section class="tool-section">
    <div class="row">
      <div class="textarea-container">
        <textarea v-model="uriInput" :placeholder="`Enter URI to ${mode}...`" rows="5"></textarea>
        <button v-if="uriInput" class="clear-button" @click="uriInput = ''" title="Clear">
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>
    </div>
    <div class="button-row">
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
    
    <div v-if="uriOutput" class="output-container">
      <div class="copy-tip">Double Click to Copy</div>
      <pre class="output" @dblclick="copyToClipboard">{{ uriOutput }}</pre>
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

.button-row {
  display: flex;
  justify-content: flex-end;
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

button {
  padding: 8px 12px;
  border: none;
  background-color: white;
  color: black;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.2s;
}

button:not(:last-child) {
  border-right: 1px solid #ddd;
}

button.active {
  background-color: #007aff;
  color: white;
}

button:hover:not(.active) {
  background-color: #f5f5f5;
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
  textarea, .button-group {
    border-color: #444;
  }
  button {
    background-color: #2a2a2a;
    color: #d4d4d4;
  }
  button:not(:last-child) {
    border-right-color: #444;
  }
  textarea {
    background-color: #2a2a2a;
    color: #d4d4d4;
  }
  button.active {
    background-color: #007aff;
    color: white;
  }
  .clear-button {
    background: rgba(255, 255, 255, 0.1);
    color: #aaa;
  }
  .clear-button:hover {
    background: rgba(255, 255, 255, 0.2);
    color: #fff;
  }
  button:hover:not(.active) {
    background-color: #3a3a3a;
  }
}
</style>
