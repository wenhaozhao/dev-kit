<script setup>
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";

const uriInput = ref("");
const uriOutput = ref("");

let debounceTimer = null;
function debounce(fn, delay = 300) {
  return (...args) => {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => fn(...args), delay);
  };
}

async function decodeUri() {
  try {
    uriOutput.value = await invoke("decode_uri", { uri: uriInput.value });
  } catch (e) {
    uriOutput.value = "Error: " + e;
  }
}

watch(uriInput, debounce(() => {
  decodeUri();
}));
</script>

<template>
  <section class="tool-section">
    <div class="row">
      <input v-model="uriInput" placeholder="Enter URI to decode..." />
    </div>
    <pre v-if="uriOutput" class="output">{{ uriOutput }}</pre>
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

input {
  flex: 1;
  padding: 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background-color: white;
  color: black;
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
  input {
    background-color: #2a2a2a;
    border-color: #444;
    color: #d4d4d4;
  }
}
</style>
