<script setup lang="ts">
import { NH4, NForm, FormInst } from "naive-ui";

import { ref } from "vue";

const formRef = ref<FormInst | null>(null);
</script>

<template>
  <div class="setting-page">
    <NForm ref="formRef" label-placement="left">
      <NH4 prefix="bar">客户端相关</NH4>
    </NForm>
  </div>
</template>

<style scoped></style>
