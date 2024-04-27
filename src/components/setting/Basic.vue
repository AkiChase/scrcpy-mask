<script setup lang="ts">
import { Store } from "@tauri-apps/plugin-store";
import { NH4, NForm, FormInst, NButton } from "naive-ui";

import { ref } from "vue";

const formRef = ref<FormInst | null>(null);
const localStore = new Store("store.bin");
</script>

<template>
  <div class="setting-page">
    <NForm ref="formRef" label-placement="left">
      <NH4 prefix="bar">客户端相关</NH4>
      <NButton @click="localStore.clear()">清除数据</NButton>
    </NForm>
  </div>
</template>

<style scoped></style>
