<script setup lang="ts">
import Basic from "./Basic.vue";
import Script from "./Script.vue";
import Mask from "./Mask.vue";
import { NTabs, NTabPane, NScrollbar } from "naive-ui";
</script>

<template>
  <div class="setting">
    <n-tabs type="line" animated placement="left" default-value="basic">
      <n-tab-pane tab="基本设置" name="basic">
        <n-scrollbar>
          <Basic />
        </n-scrollbar>
      </n-tab-pane>
      <n-tab-pane tab="蒙版设置" name="mask">
        <n-scrollbar>
          <Mask />
        </n-scrollbar>
      </n-tab-pane>
      <n-tab-pane tab="脚本设置" name="script">
        <n-scrollbar>
          <Script />
        </n-scrollbar>
      </n-tab-pane>
    </n-tabs>
  </div>
</template>

<style scoped lang="scss">
.setting {
  background-color: var(--content-bg-color);
  color: var(--light-color);
  overflow: hidden;
  display: flex;

  .n-tab-pane {
    padding: 0;
  }

  .setting-page {
    padding: 10px 25px;
  }
}
</style>
