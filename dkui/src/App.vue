<script setup>
import { ref } from "vue";
import JsonParser from "./components/JsonParser.vue";
import JsonDiff from "./components/JsonDiff.vue";
import UriTools from "./components/UriTools.vue";
import TimeTools from "./components/TimeTools.vue";

const jsonInput = ref("");
const jsonRightInput = ref("");
const jsonQuery = ref("");
const currentTab = ref("json");

function updateJson(val) {
  jsonInput.value = val;
}

function updateRightJson(val) {
  jsonRightInput.value = val;
}

function updateQuery(val) {
  jsonQuery.value = val;
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

    <UriTools v-if="currentTab === 'uri'" />

    <JsonParser v-if="currentTab === 'json'" 
      :initialJson="jsonInput" 
      :initialQuery="jsonQuery"
      @update:json="updateJson"
      @update:query="updateQuery" />

    <JsonDiff v-if="currentTab === 'diff'" 
      :initialLeftJson="jsonInput"
      :initialRightJson="jsonRightInput"
      :initialQuery="jsonQuery"
      @update:leftJson="updateJson"
      @update:rightJson="updateRightJson"
      @update:query="updateQuery" />

    <TimeTools v-if="currentTab === 'time'" />
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
}
</style>
