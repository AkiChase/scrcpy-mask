<script setup lang="ts">
import {
  GameControllerOutline,
  LogoAndroid,
  SettingsOutline,
  ReturnDownBackOutline,
  VolumeHighOutline,
  VolumeLowOutline,
  StopOutline,
  ListOutline,
  BulbOutline,
  Bulb,
} from "@vicons/ionicons5";
import { Keyboard24Regular } from "@vicons/fluent";
import { useMessage } from "naive-ui";
import { useGlobalStore } from "../../store/global";
import { sendSetScreenPowerMode } from "../../frontcommand/controlMsg";
import { AndroidKeycode } from "../../frontcommand/android";
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { SendKeyAction, sendKey } from "../../frontcommand/scrcpyMaskCmd";
import ModuleNav from "./ModuleNav.vue";
import FuncButton from "./FuncButton.vue";

const { t } = useI18n();
const store = useGlobalStore();
const message = useMessage();

const nextScreenPowerMode = ref(0);

async function sendKeyCodeToDevice(code: AndroidKeycode) {
  if (store.controledDevice) {
    await sendKey({
      action: SendKeyAction.Default,
      keycode: code,
    });
  } else {
    message.error(t("sidebar.noControledDevice"));
  }
}

async function changeScreenPowerMode() {
  if (store.controledDevice) {
    sendSetScreenPowerMode({ mode: nextScreenPowerMode.value });
    nextScreenPowerMode.value = nextScreenPowerMode.value ? 0 : 2;
  } else {
    message.error(t("sidebar.noControledDevice"));
  }
}
</script>

<template>
  <div class="sidebar">
    <div data-tauri-drag-region class="logo">S M</div>
    <div class="module-nav">
      <ModuleNav
        name="mask"
        :icon="GameControllerOutline"
        :tip="t('sidebar.tip.mask')"
      />
      <ModuleNav
        name="device"
        :icon="LogoAndroid"
        :tip="t('sidebar.tip.device')"
      />
      <ModuleNav
        name="keyboard"
        :icon="Keyboard24Regular"
        :tip="t('sidebar.tip.keyboard')"
      />
      <ModuleNav
        name="setting"
        :icon="SettingsOutline"
        :tip="t('sidebar.tip.setting')"
      />
    </div>
    <div data-tauri-drag-region class="drag" />
    <div class="func-btn">
      <FuncButton
        @click="changeScreenPowerMode"
        :tip="nextScreenPowerMode? t('sidebar.tip.screenPowerModeOff'): t('sidebar.tip.screenPowerModeOn')"
        :icon="nextScreenPowerMode ? Bulb : BulbOutline"
      />
      <FuncButton
        @click="sendKeyCodeToDevice(AndroidKeycode.AKEYCODE_VOLUME_UP)"
        :tip="t('sidebar.tip.volumeUp')"
        :icon="VolumeHighOutline"
      />
      <FuncButton
        @click="sendKeyCodeToDevice(AndroidKeycode.AKEYCODE_VOLUME_DOWN)"
        :tip="t('sidebar.tip.volumeDown')"
        :icon="VolumeLowOutline"
      />
      <FuncButton
        @click="sendKeyCodeToDevice(AndroidKeycode.AKEYCODE_BACK)"
        :tip="t('sidebar.tip.back')"
        :icon="ReturnDownBackOutline"
      />
      <FuncButton
        @click="sendKeyCodeToDevice(AndroidKeycode.AKEYCODE_HOME)"
        :tip="t('sidebar.tip.home')"
        :icon="StopOutline"
      />
      <FuncButton
        @click="sendKeyCodeToDevice(AndroidKeycode.AKEYCODE_APP_SWITCH)"
        :tip="t('sidebar.tip.appSwitch')"
        :icon="ListOutline"
      />
    </div>
  </div>
</template>

<style scoped lang="scss">
.sidebar {
  background-color: var(--bg-color);
  border-right: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 10px 0 0 10px;
  grid-area: sidebar;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  user-select: none;
  -webkit-user-select: none;

  .logo {
    height: 30px;
    font-size: 18px;
    font-weight: bold;
    display: flex;
    justify-content: center;
    align-items: center;
    color: var(--light-color);
    cursor: pointer;
  }

  .drag {
    flex-grow: 1;
    width: 100%;
  }

  .module-nav {
    display: flex;
    flex-direction: column;
  }

  .func-btn {
    color: var(--light-color);
    font-size: 20px;
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
  }
}
</style>
