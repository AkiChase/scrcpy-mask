<script setup lang="ts">
import {
  NFlex,
  NH4,
  NButton,
  NIcon,
  NP,
  useMessage,
  useDialog,
} from "naive-ui";
import { LogoGithub } from "@vicons/ionicons5";
import { open } from "@tauri-apps/plugin-shell";
import { fetch } from "@tauri-apps/plugin-http";
import { getVersion } from "@tauri-apps/api/app";
import { h, onMounted, ref } from "vue";
import { useGlobalStore } from "../../store/global";

const message = useMessage();
const dialog = useDialog();
const store = useGlobalStore();

const appVersion = ref("");
onMounted(async () => {
  appVersion.value = await getVersion();
});

function opendWebsite(url: string) {
  open(url);
}

function renderUpdateInfo(content: string) {
  const pList = content.split("\r\n").map((line: string) => h("p", line));
  return h("div", { style: "margin: 20px 0" }, pList);
}

async function checkUpdate() {
  store.showLoading();
  try {
    const res = await fetch(
      "https://api.github.com/repos/AkiChase/scrcpy-mask/releases/latest",
      {
        connectTimeout: 5000,
      }
    );
    store.hideLoading();
    if (res.status !== 200) {
      message.error("检查更新失败");
    } else {
      const data = await res.json();
      const latestVersion = (data.tag_name as string).slice(1);
      if (latestVersion <= appVersion.value) {
        message.success(`最新版本: ${latestVersion}，当前已是最新版本`);
        return;
      }
      const body = data.body as string;
      dialog.info({
        title: `最新版本：${data.tag_name}`,
        content: () => renderUpdateInfo(body),
        positiveText: "前往发布页",
        negativeText: "取消",
        onPositiveClick: () => {
          opendWebsite(data.html_url);
        },
      });
    }
  } catch (e) {
    store.hideLoading();
    console.error(e);
    message.error("检查更新失败");
  }
}
</script>

<template>
  <div class="setting-page">
    <NH4 prefix="bar">关于</NH4>
    <NP
      >A Scrcpy client in Rust & Tarui aimed at providing mouse and key mapping
      to control Android device.</NP
    >
    <NFlex class="website">
      <NButton
        text
        @click="opendWebsite('https://github.com/AkiChase/scrcpy-mask')"
      >
        <template #icon>
          <NIcon><LogoGithub /> </NIcon>
        </template>
        Github repo
      </NButton>
      <NButton
        text
        @click="opendWebsite('https://space.bilibili.com/440760180')"
      >
        <template #icon>
          <NIcon
            ><svg
              t="1714874601502"
              class="icon"
              viewBox="0 0 1024 1024"
              version="1.1"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path
                d="M977.2 208.2c33.4 36.2 48.8 79.4 46.6 131.4v404.8c-0.8 52.8-18.4 96.2-53 130.2-34.4 34-78.2 51.8-131 53.4H184.04c-52.9-1.6-96.42-19.6-130.56-54.4C19.364 838.8 1.534 793 0 736.4V339.6c1.534-52 19.364-95.2 53.48-131.4C87.62 175.5 131.14 157.54 184.04 156h58.76L192.1 104.38c-11.5-11.46-17.26-26-17.26-43.58 0-17.6 5.76-32.12 17.26-43.594C203.6 5.736 218.2 0 235.8 0s32.2 5.736 43.8 17.206L426.2 156h176l149-138.794C763.4 5.736 778.4 0 796 0c17.6 0 32.2 5.736 43.8 17.206 11.4 11.474 17.2 25.994 17.2 43.594 0 17.58-5.8 32.12-17.2 43.58L789.2 156h58.6c52.8 1.54 96 19.5 129.4 52.2z m-77.6 139.4c-0.8-19.2-7.4-34.8-21.4-47-10.4-12.2-28-18.8-45.4-19.6H192.1c-19.18 0.8-34.9 7.4-47.16 19.6-12.28 12.2-18.8 27.8-19.56 47v388.8c0 18.4 6.52 34 19.56 47s28.76 19.6 47.16 19.6H832.8c18.4 0 34-6.6 46.6-19.6 12.6-13 19.4-28.6 20.2-47V347.6z m-528.6 85.4c12.6 12.6 19.4 28.2 20.2 46.4V546c-0.8 18.4-7.4 33.8-19.6 46.4-12.4 12.6-28 19-47.2 19-19.2 0-35-6.4-47.2-19-12.2-12.6-18.8-28-19.6-46.4v-66.6c0.8-18.2 7.6-33.8 20.2-46.4 12.6-12.6 26.4-19.2 46.6-20 18.4 0.8 34 7.4 46.6 20z m383 0c12.6 12.6 19.4 28.2 20.2 46.4V546c-0.8 18.4-7.4 33.8-19.6 46.4-12.2 12.6-28 19-47.2 19-19.2 0-34.8-6.4-47.2-19-14-12.6-18.8-28-19.4-46.4v-66.6c0.6-18.2 7.4-33.8 20-46.4 12.6-12.6 28.2-19.2 46.6-20 18.4 0.8 34 7.4 46.6 20z"
                p-id="4281"
              ></path>
            </svg>
          </NIcon>
        </template>
        BiliBili
      </NButton>
    </NFlex>
    <NH4 prefix="bar">更新</NH4>
    <NP>当前版本：{{ appVersion }}</NP>
    <NButton @click="checkUpdate">检查更新</NButton>
  </div>
</template>

<style scoped>
.setting-page {
  padding: 10px 25px;

  .website .n-button {
    margin-right: 30px;
  }
}
</style>
