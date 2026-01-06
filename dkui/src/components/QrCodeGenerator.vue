<script setup>
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";

const content = ref("");
const ecLevel = ref("q");
const version = ref(3);
const outputType = ref("image");
const qrData = ref("");
const error = ref("");

const ecLevels = [
  { label: "7% (Low)", value: "l" },
  { label: "15% (Medium)", value: "m" },
  { label: "25% (Quartile)", value: "q" },
  { label: "30% (High)", value: "h" },
];

const outputTypes = [
  { label: "SVG", value: "svg" },
  { label: "PNG", value: "image" },
];

async function generateQrCode() {
  if (!content.value) {
    qrData.value = "";
    error.value = "";
    return;
  }

  try {
    error.value = "";
    const res = await invoke("generate_qrcode", {
      content: content.value,
      ecLevel: ecLevel.value,
      version: version.value,
      outputType: outputType.value,
    });
    qrData.value = res;
  } catch (e) {
    error.value = e.toString();
    qrData.value = "";
  }
}

// Debounce generation
let timer = null;
watch([content, ecLevel, version, outputType], () => {
  if (timer) clearTimeout(timer);
  timer = setTimeout(generateQrCode, 300);
});

async function saveToFile() {
  if (!qrData.value) return;

  const isSvg = outputType.value === "svg";
  const filePath = await save({
    filters: isSvg
      ? [{ name: "SVG Image", extensions: ["svg"] }]
      : [{ name: "PNG Image", extensions: ["png"] }],
    defaultPath: isSvg ? "qrcode.svg" : "qrcode.png",
  });

  if (!filePath) return;

  try {
    if (isSvg) {
      await invoke("save_to_file", { path: filePath, content: qrData.value });
    } else {
      await invoke("save_image_to_file", { path: filePath, base64Content: qrData.value });
    }
  } catch (e) {
    error.value = `Failed to save file: ${e.toString()}`;
  }
}
</script>

<template>
  <section class="tool-section">
    <div class="input-section">
      <textarea
        v-model="content"
        placeholder="Enter text to generate QR code..."
        class="qr-input"
        rows="5"
      ></textarea>
      
      <div class="controls row">
        <div class="control-group">
          <label>Error Correction:</label>
          <select v-model="ecLevel">
            <option v-for="level in ecLevels" :key="level.value" :value="level.value">
              {{ level.label }}
            </option>
          </select>
        </div>
        
        <div class="control-group">
          <label>Version (1-40):</label>
          <input type="number" v-model.number="version" min="1" max="40" />
        </div>

        <div class="control-group">
          <label>Output Type:</label>
          <select v-model="outputType">
            <option v-for="type in outputTypes" :key="type.value" :value="type.value">
              {{ type.label }}
            </option>
          </select>
        </div>
      </div>
    </div>

    <div class="output-preview">
      <div v-if="error" class="error-message">
        {{ error }}
      </div>
      <div v-else-if="qrData" class="qr-display">
        <div v-if="outputType === 'svg'" class="svg-container" v-html="qrData"></div>
        <div v-else class="png-container">
          <img :src="qrData" alt="QR Code" />
        </div>
        <button @click="saveToFile" class="save-btn">Save to File</button>
      </div>
      <div v-else class="placeholder">
        QR code preview
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

.input-section {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 15px;
  margin-bottom: 20px;
}

.qr-input {
  width: 100%;
  padding: 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
  resize: vertical;
  font-family: inherit;
  box-sizing: border-box;
  min-height: calc(1.5em * 5 + 16px);
}

.row {
  display: flex;
  gap: 10px;
  margin-top: 10px;
  flex-wrap: wrap;
  align-items: center;
  justify-content: center;
}

.control-group {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.9em;
}

select,
input[type="number"] {
  padding: 6px;
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

.output-preview {
  width: 100%;
  min-height: 240px;
  display: flex;
  justify-content: center;
  align-items: center;
  border: 1px dashed #ccc;
  border-radius: 4px;
  background-color: #f9f9f9;
  flex-shrink: 0;
  padding: 20px;
  box-sizing: border-box;
}

.qr-display {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 15px;
}

.save-btn {
  margin-top: 10px;
}

.svg-container,
.png-container {
  background: white;
  padding: 5px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.svg-container :deep(svg),
.png-container img {
  max-width: 200px;
  max-height: 200px;
  display: block;
}

.error-message {
  color: #d32f2f;
  background-color: #fdecea;
  padding: 10px;
  border-radius: 4px;
  font-size: 0.8em;
  word-break: break-all;
}

.placeholder {
  color: #999;
}

@media (prefers-color-scheme: dark) {
  .tool-section {
    border-color: #444;
  }
  .qr-input, select, input[type="number"] {
    background-color: #2a2a2a;
    border-color: #444;
    color: #d4d4d4;
  }
  .output-preview {
    background-color: #252525;
    border-color: #444;
  }
  .svg-container,
  .png-container {
    background: #fff; /* Keep QR code white for readability */
  }
}
</style>
