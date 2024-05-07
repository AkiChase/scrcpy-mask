<script setup lang="ts">
import Basic from "./Basic.vue";
import Script from "./Script.vue";
import Mask from "./Mask.vue";
import About from "./About.vue";
import { NTabs, NTabPane, NScrollbar, NSpin } from "naive-ui";
import { useGlobalStore } from "../../store/global";

const store = useGlobalStore();
</script>

<template>
  <div class="setting">
    <NSpin :show="store.showLoadingRef">
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
        <NTabPane tab="关于" name="about">
          <NScrollbar>
            <About />
          </NScrollbar>
        </NTabPane>
      </NTabs>
    </NSpin>
  </div>
</template>

<style scoped lang="scss">
.setting {
  background-color: var(--content-bg-color);
  color: var(--light-color);
  height: 100%;
  overflow-y: auto;
  display: flex;

  .n-tabs{
    height: 100%;
  }

  .n-tab-pane {
    padding: 0;
  }

  .setting-page {
    padding: 10px 25px;
  }
}
</style>
