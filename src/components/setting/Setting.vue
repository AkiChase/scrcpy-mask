<script setup lang="ts">
import Basic from "./Basic.vue";
import Mask from "./Mask.vue";
import Data from "./Data.vue";
import About from "./About.vue";
import { NTabs, NTabPane, NScrollbar, NSpin } from "naive-ui";
import { useGlobalStore } from "../../store/global";

const store = useGlobalStore();
</script>

<template>
  <div class="setting">
    <NSpin :show="store.showLoadingRef">
      <NTabs type="line" animated placement="left" default-value="basic">
        <NTabPane :tab="$t('pages.Setting.tabs.basic')" name="basic">
          <NScrollbar>
            <Basic />
          </NScrollbar>
        </NTabPane>
        <NTabPane :tab="$t('pages.Setting.tabs.mask')" name="mask">
          <NScrollbar>
            <Mask />
          </NScrollbar>
        </NTabPane>
        <NTabPane :tab="$t('pages.Setting.tabs.data')" name="data">
          <NScrollbar>
            <Data />
          </NScrollbar>
        </NTabPane>
        <NTabPane :tab="$t('pages.Setting.tabs.about')" name="about">
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

  .n-tabs {
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
