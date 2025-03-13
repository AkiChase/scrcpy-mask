<script setup lang="ts">
import Basic from "./Basic.vue";
import Mask from "./Mask.vue";
import Data from "./Data.vue";
import About from "./About.vue";
import { NTabs, NTabPane, NSpin } from "naive-ui";
import { useGlobalStore } from "../../store/global";
import SettingTab from "./SettingTab.vue";
import { useHorRotation } from "../../tools/hooks";
import { onActivated } from "vue";

// TODO Switch back to landscape size when entering Settings and Devices screen
const store = useGlobalStore();
const horRotation = useHorRotation();

onActivated(() => {
  horRotation();
});
</script>

<template>
  <div class="setting">
    <NSpin :show="store.showLoadingFlag">
      <NTabs type="line" animated placement="left" default-value="basic">
        <NTabPane :tab="$t('pages.Setting.tabs.basic')" name="basic">
          <SettingTab>
            <Basic />
          </SettingTab>
        </NTabPane>
        <NTabPane :tab="$t('pages.Setting.tabs.mask')" name="mask">
          <SettingTab>
            <Mask />
          </SettingTab>
        </NTabPane>
        <NTabPane :tab="$t('pages.Setting.tabs.data')" name="data">
          <SettingTab>
            <Data />
          </SettingTab>
        </NTabPane>
        <NTabPane :tab="$t('pages.Setting.tabs.about')" name="about">
          <SettingTab>
            <About />
          </SettingTab>
        </NTabPane>
      </NTabs>
    </NSpin>
  </div>
</template>

<style scoped lang="scss">
@use "../../css/common.scss";

.setting {
  @include common.contentContainer;
  background-color: var(--content-bg-color);
  color: var(--light-color);
  overflow-y: auto;
  overflow-x: hidden;
  // for spin div
  display: flex;
  flex-direction: column;

  .n-tabs {
    height: 100%;
  }
  .n-tab-pane {
    padding: 0;
  }
}
</style>
