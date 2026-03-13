<script setup>
import { ref, watch } from "vue";

const dividend = ref("");
const divisor = ref("");
const quotient = ref("");
const remainder = ref("");
const error = ref("");

function clearAll() {
  dividend.value = "";
  divisor.value = "";
  quotient.value = "";
  remainder.value = "";
  error.value = "";
}

function calculate() {
  error.value = "";
  quotient.value = "";
  remainder.value = "";

  const a = dividend.value.trim();
  const b = divisor.value.trim();

  if (!a || !b) {
    error.value = "请输入被除数和除数";
    return;
  }

  if (!/^-?\d+$/.test(a) || !/^-?\d+$/.test(b)) {
    error.value = "仅支持整数计算";
    return;
  }

  try {
    const left = BigInt(a);
    const right = BigInt(b);

    if (right === 0n) {
      error.value = "除数不能为 0";
      return;
    }

    quotient.value = (left / right).toString();
    remainder.value = (left % right).toString();
  } catch (e) {
    error.value = `计算失败: ${e}`;
  }
}

watch([dividend, divisor], () => {
  if (!dividend.value.trim() && !divisor.value.trim()) {
    quotient.value = "";
    remainder.value = "";
    error.value = "";
    return;
  }
  calculate();
});
</script>

<template>
  <section class="tool-section">
    <div class="form-row">
      <label for="dividend">被除数:</label>
      <input id="dividend" v-model="dividend" type="text" placeholder="请输入被除数" />
    </div>

    <div class="form-row">
      <label for="divisor">除数:</label>
      <input id="divisor" v-model="divisor" type="text" placeholder="请输入除数" />
    </div>

    <div class="actions">
      <button class="primary-btn" @click="clearAll">清除</button>
    </div>

    <div class="form-row">
      <label for="quotient">商:</label>
      <input id="quotient" :value="quotient" type="text" readonly />
    </div>

    <div class="form-row">
      <label for="remainder">余数:</label>
      <input id="remainder" :value="remainder" type="text" readonly />
    </div>

    <p v-if="error" class="error-text">{{ error }}</p>
  </section>
</template>

<style scoped>
.tool-section {
  margin: 0 auto 30px;
  padding: 20px;
  border: 1px solid #ccc;
  border-radius: 8px;
  text-align: left;
  max-width: 560px;
}

.form-row {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 18px;
  justify-content: center;
}

label {
  width: 90px;
  font-size: 18px;
}

input {
  flex: 1;
  padding: 10px 14px;
  border: 1px solid #c9c9c9;
  border-radius: 6px;
  font-size: 34px;
  line-height: 1.2;
  background-color: #fff;
  color: #333;
}

input[readonly] {
  background-color: #f2f2f2;
}

.actions {
  display: flex;
  gap: 8px;
  margin-bottom: 18px;
  width: 100%;
  justify-content: flex-end;
}

.primary-btn {
  border: none;
  border-radius: 6px;
  background-color: #2f7fc7;
  color: #fff;
  padding: 10px 16px;
  font-size: 16px;
  cursor: pointer;
}

.primary-btn:hover {
  background-color: #2569a6;
}

.error-text {
  text-align: center;
  color: #b91c1c;
}

:root.dark-mode input {
  background-color: #252525;
  color: #ddd;
  border-color: #555;
}

:root.dark-mode input[readonly] {
  background-color: #333;
}
</style>
