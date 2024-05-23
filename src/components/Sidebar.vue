<script setup lang="ts">
import { useRouter, useRoute } from "vue-router";
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
import { NIcon, useMessage } from "naive-ui";
import { useGlobalStore } from "../store/global";
import { sendSetScreenPowerMode } from "../frontcommand/controlMsg";
import { AndroidKeycode } from "../frontcommand/android";
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { SendKeyAction, sendKey } from "../frontcommand/scrcpyMaskCmd";

const { t } = useI18n();
const router = useRouter();
const route = useRoute();
const store = useGlobalStore();
const message = useMessage();

const nextScreenPowerMode = ref(0);

function nav(name: string) {
  router.replace({ name });
}

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
    <div class="module">
      <div :class="{ active: route.name == 'mask' }" @click="nav('mask')">
        <NIcon>
          <GameControllerOutline />
        </NIcon>
      </div>
      <div :class="{ active: route.name == 'device' }" @click="nav('device')">
        <NIcon>
          <LogoAndroid />
        </NIcon>
      </div>
      <div
        :class="{ active: route.name == 'keyboard' }"
        @click="nav('keyboard')"
      >
        <NIcon>
          <Keyboard24Regular />
        </NIcon>
      </div>
      <div :class="{ active: route.name == 'setting' }" @click="nav('setting')">
        <NIcon>
          <SettingsOutline />
        </NIcon>
      </div>
    </div>
    <div data-tauri-drag-region class="drag"></div>
    <div class="nav">
      <div @click="changeScreenPowerMode">
        <NIcon>
          <Bulb v-if="nextScreenPowerMode" />
          <BulbOutline v-else />
        </NIcon>
      </div>
      <div @click="sendKeyCodeToDevice(AndroidKeycode.AKEYCODE_VOLUME_UP)">
        <NIcon>
          <VolumeHighOutline />
        </NIcon>
      </div>
      <div @click="sendKeyCodeToDevice(AndroidKeycode.AKEYCODE_VOLUME_DOWN)">
        <NIcon>
          <VolumeLowOutline />
        </NIcon>
      </div>
      <div @click="sendKeyCodeToDevice(AndroidKeycode.AKEYCODE_BACK)">
        <NIcon>
          <ReturnDownBackOutline />
        </NIcon>
      </div>
      <div @click="sendKeyCodeToDevice(AndroidKeycode.AKEYCODE_HOME)">
        <NIcon>
          <StopOutline />
        </NIcon>
      </div>
      <div @click="sendKeyCodeToDevice(AndroidKeycode.AKEYCODE_APP_SWITCH)">
        <NIcon>
          <ListOutline />
        </NIcon>
      </div>
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

  .drag{
    flex-grow: 1;
    width: 100%;
  }

  .module {
    display: flex;
    flex-direction: column;

    & > div {
      flex-shrink: 0;
      height: 50px;
      color: var(--gray-color);
      display: flex;
      align-items: center;
      justify-content: center;
      transition: transform 0.3s ease;
      box-sizing: border-box;
      font-size: 28px;
      cursor: pointer;

      &:hover {
        color: var(--primary-hover-color);
        transform: scale(1.05);
      }
      &:active {
        color: var(--primary-pressed-color);
        transform: scale(0.9);
      }
    }

    & > div.active {
      color: var(--primary-color);
      border-left: 3px solid var(--primary-color);
      border-radius: 3px;
    }
  }

  .nav {
    color: var(--light-color);
    font-size: 20px;
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
    & > div {
      height: 40px;
      display: flex;
      justify-content: center;
      align-items: center;

      cursor: pointer;
      transition: transform 0.3s ease;

      &:hover {
        color: var(--primary-hover-color);
        transform: scale(1.1);
      }
      &:active {
        color: var(--primary-pressed-color);
        transform: scale(0.9);
      }
    }
  }
}
</style>
