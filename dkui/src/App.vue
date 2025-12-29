<script setup>
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

const uriInput = ref("");
const uriOutput = ref("");
const jsonInput = ref("");
const jsonRightInput = ref("");
const jsonOutput = ref("");
const jsonQuery = ref("");
const diffTool = ref("");
const availableDiffTools = ref([]);
const timeOutput = ref("");

const currentTab = ref("uri");

onMounted(async () => {
  try {
    availableDiffTools.value = await invoke("get_available_diff_tools");
    if (availableDiffTools.value.length > 0) {
      diffTool.value = availableDiffTools.value[0];
    }
  } catch (e) {
    console.error("Failed to fetch diff tools:", e);
  }
});

async function decodeUri() {
  try {
    uriOutput.value = await invoke("decode_uri", { uri: uriInput.value });
  } catch (e) {
    uriOutput.value = "Error: " + e;
  }
}

async function formatJson() {
  try {
    jsonOutput.value = await invoke("format_json", { json: jsonInput.value });
  } catch (e) {
    jsonOutput.value = "Error: " + e;
  }
}

async function queryJson() {
  try {
    const res = await invoke("query_json", { json: jsonInput.value, query: jsonQuery.value });
    jsonOutput.value = res.join("\n");
  } catch (e) {
    jsonOutput.value = "Error: " + e;
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

async function getNow() {
  try {
    timeOutput.value = await invoke("now_time", { timezone: null, format: null });
  } catch (e) {
    timeOutput.value = "Error: " + e;
  }
}
</script>

<template>
  <main class="container">
    <h1>DevKit UI</h1>

    <div class="tabs">
      <button :class="{ active: currentTab === 'uri' }" @click="currentTab = 'uri'">URI Tools</button>
      <button :class="{ active: currentTab === 'json' }" @click="currentTab = 'json'">JSON Tools</button>
      <button :class="{ active: currentTab === 'diff' }" @click="currentTab = 'diff'">JSON Diff</button>
      <button :class="{ active: currentTab === 'time' }" @click="currentTab = 'time'">Time Tools</button>
    </div>

    <section v-if="currentTab === 'uri'" class="tool-section">
      <h3>URI Decoder</h3>
      <div class="row">
        <input v-model="uriInput" placeholder="Enter URI to decode..." />
        <button @click="decodeUri">Decode</button>
      </div>
      <pre v-if="uriOutput" class="output">{{ uriOutput }}</pre>
    </section>

    <section v-if="currentTab === 'json'" class="tool-section">
      <h3>JSON Tools</h3>
      <div class="json-inputs">
        <textarea v-model="jsonInput" placeholder="Enter JSON..." rows="10"></textarea>
      </div>
      <div class="row">
        <button @click="formatJson">Format</button>
        <input v-model="jsonQuery" placeholder="JSON Path (e.g. $.a.b)" />
        <button @click="queryJson">Query</button>
      </div>
      <pre v-if="jsonOutput" class="output">{{ jsonOutput }}</pre>
    </section>

    <section v-if="currentTab === 'diff'" class="tool-section">
      <h3>JSON Diff</h3>
      <div class="json-inputs">
        <textarea v-model="jsonInput" placeholder="Enter JSON (Left)..." rows="10"></textarea>
        <textarea v-model="jsonRightInput" placeholder="Enter JSON (Right)..." rows="10"></textarea>
      </div>
      <div class="row">
        <input v-model="jsonQuery" placeholder="JSON Path (e.g. $.a.b)" />
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
      </div>
      <pre v-if="jsonOutput" class="output">{{ jsonOutput }}</pre>
    </section>

    <section v-if="currentTab === 'time'" class="tool-section">
      <h3>Time Tools</h3>
      <div class="row">
        <button @click="getNow">Get Current Time (Local)</button>
      </div>
      <p v-if="timeOutput" class="output">{{ timeOutput }}</p>
    </section>
  </main>
</template>

<style scoped>
.container {
  max-width: 800px;
  margin: 0 auto;
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

.json-inputs {
  display: flex;
  gap: 10px;
  margin-bottom: 10px;
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
}
</style>
