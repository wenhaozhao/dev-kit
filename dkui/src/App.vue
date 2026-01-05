<script setup>
import {onMounted, ref} from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import { invoke } from "@tauri-apps/api/core";
import { message } from "@tauri-apps/plugin-dialog";
import JsonParser from "./components/JsonParser.vue";
import JsonDiff from "./components/JsonDiff.vue";
import UriParser from "./components/UriParser.vue";
import UriDecoder from "./components/UriDecoder.vue";
import TimeParser from "./components/TimeParser.vue";

const jsonInput = ref("");
const jsonRightInput = ref("");
const jsonQuery = ref("");
const currentTab = ref("json");

const is_show_add_to_path_bth = ref(false);
const openGithub = async () => {
  await openUrl("https://github.com/wenhaozhao/dev-kit");
};

const show_add_to_path_bth = async () => {
  try {
    const res = await invoke("show_add_to_path_bth");
    is_show_add_to_path_bth.value = typeof res === 'string' && res.length === 0;
  } catch (err) {
  }
}

onMounted(() => {
  show_add_to_path_bth();
});

const addToPath = async () => {
  try {
    const res = await invoke("add_to_path");
    await message(res, { title: 'Success', kind: 'info' });
    await show_add_to_path_bth()
  } catch (err) {
    await message(err, { title: 'Error', kind: 'error' });
  }
};

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
      <button :class="{ active: currentTab === 'uri_parse' }" @click="currentTab = 'uri_parse'">URI Parser</button>
      <button :class="{ active: currentTab === 'uri_decoder' }" @click="currentTab = 'uri_decoder'">URI Decoder</button>
      <button :class="{ active: currentTab === 'time' }" @click="currentTab = 'time'">Time Parser</button>
      <a href="#" class="github-link" @click.prevent="addToPath" title="Add devkit(dk) to Path" v-if="is_show_add_to_path_bth">
        <svg height="24" viewBox="0 0 24 24" width="24" fill="currentColor">
          <path d="M20,19V7H4V19H20M20,3A2,2 0 0,1 22,5V19A2,2 0 0,1 20,21H4A2,2 0 0,1 2,19V5C2,3.89 2.9,3 4,3H20M13,17V15H18V17H13M9.58,13L5.57,9H8.4L11.7,12.3C12.09,12.69 12.09,13.33 11.7,13.72L8.42,17H5.59L9.58,13Z" />
        </svg>
      </a>
      <a href="#" class="github-link" @click.prevent="openGithub" title="GitHub">
        <svg height="24" viewBox="0 0 16 16" width="24" fill="currentColor">
          <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"></path>
        </svg>
      </a>
    </div>

    <KeepAlive>
      <UriParser v-if="currentTab === 'uri_parse'" />
    </KeepAlive>

    <KeepAlive>
      <UriDecoder v-if="currentTab === 'uri_decoder'" />
    </KeepAlive>

    <KeepAlive>
      <JsonParser v-if="currentTab === 'json'" 
        :initialJson="jsonInput" 
        :initialQuery="jsonQuery"
        @update:json="updateJson"
        @update:query="updateQuery" />
    </KeepAlive>

    <KeepAlive>
      <JsonDiff v-if="currentTab === 'diff'" 
        :initialLeftJson="jsonInput"
        :initialRightJson="jsonRightInput"
        :initialQuery="jsonQuery"
        @update:leftJson="updateJson"
        @update:rightJson="updateRightJson"
        @update:query="updateQuery" />
    </KeepAlive>

    <KeepAlive>
      <TimeParser v-if="currentTab === 'time'" />
    </KeepAlive>
  </main>
</template>

<style scoped>
.container {
  width: 100%;
  box-sizing: border-box;
  margin: 0;
  padding: 0 20px 20px 20px;
}

.tabs {
  display: flex;
  gap: 10px;
  margin-bottom: 20px;
  border-bottom: 1px solid #ccc;
  padding-bottom: 10px;
  position: sticky;
  top: 0;
  background-color: #fff;
  z-index: 100;
  padding-top: 10px;
  overflow-x: auto;
  scrollbar-width: none; /* Firefox */
  align-items: center;
}

.tabs::-webkit-scrollbar {
  display: none; /* Safari and Chrome */
}

.github-link {
  display: flex;
  align-items: center;
  color: #333;
  transition: color 0.2s;
  padding: 0 10px;
}

.github-link:first-of-type {
  margin-left: auto;
}

.github-link:hover {
  color: #000;
}

.tabs button {
  background: none;
  color: #666;
  border: 1px solid transparent;
  padding: 8px 16px;
  cursor: pointer;
  white-space: nowrap;
}

.tabs button.active {
  background-color: #396cd8;
  color: white;
  border-radius: 4px;
}

@media (prefers-color-scheme: dark) {
  .tabs {
    border-color: #444;
    background-color: #1a1a1a;
  }
  .tabs button {
    color: #aaa;
  }
  .tabs button.active {
    color: white;
  }
  .github-link {
    color: #ccc;
  }
  .github-link:hover {
    color: #fff;
  }
}
</style>
