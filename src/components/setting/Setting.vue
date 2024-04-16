<script setup lang="ts">
import Basic from "./Basic.vue";
import Script from "./Script.vue";
import Mask from "./Mask.vue";
import { NTabs, NTabPane, NScrollbar } from "naive-ui";
</script>

<template>
  <div class="setting">
    <NTabs type="line" animated placement="left" default-value="basic">
      <NTabPane tab="基本设置" name="basic">
        <NScrollbar>
          <Basic />
        </NScrollbar>
      </NTabPane>
      <NTabPane tab="蒙版设置" name="mask">
        <NScrollbar>
          <Mask />
        </NScrollbar>
      </NTabPane>
      <NTabPane tab="脚本设置" name="script">
        <NScrollbar>
          <Script />
        </NScrollbar>
      </NTabPane>
    </NTabs>
  </div>
</template>

<style scoped lang="scss">
.setting {
  background-color: var(--content-bg-color);
  color: var(--light-color);
  height: 100%;
  overflow-y: auto;
  display: flex;

  .NTabPane {
    padding: 0;
  }

  .setting-page {
    padding: 10px 25px;
  }
}
</style>
