<script setup lang="ts">
import { useRouter, useRoute } from "vue-router";
import {
  GameControllerOutline,
  LogoAndroid,
  SettingsOutline,
  ReturnDownBackOutline,
  StopOutline,
  ListOutline,
} from "@vicons/ionicons5";
import { Keyboard24Regular } from "@vicons/fluent";
import { NIcon } from "naive-ui";

const router = useRouter();
const route = useRoute();

function nav(name: string) {
  router.replace({ name });
}

</script>

<template>
  <div data-tauri-drag-region class="sidebar">
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

    <div class="nav">
      <div>
        <NIcon>
          <ReturnDownBackOutline />
        </NIcon>
      </div>
      <div>
        <NIcon>
          <StopOutline />
        </NIcon>
      </div>
      <div>
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

  .logo {
    height: 30px;
    font-size: 18px;
    font-weight: bold;
    display: flex;
    justify-content: center;
    align-items: center;
    color: var(--light-color);
  }

  .module {
    display: flex;
    flex: 1;
    min-height: 0;

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

      .NIcon {
        cursor: pointer;
        transition: transform 0.3s ease;
      }

      .NIcon:hover {
        color: var(--primary-hover-color);
        transform: scale(1.1);
      }
      .NIcon:active {
        color: var(--primary-pressed-color);
        transform: scale(0.9);
      }
    }
  }
}
</style>
