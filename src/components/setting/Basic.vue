<script setup lang="ts">
import { NButton } from "naive-ui";
import {
  adbDevices,
  reverseServerPort,
  pushServerFile,
  openSocketServer,
  startScrcpyServer,
  closeSocketServer,
} from "../../invoke";
import { onMounted, onUnmounted } from "vue";
import { UnlistenFn, listen } from "@tauri-apps/api/event";

async function test() {
  let devices = await adbDevices();
  if (devices.length == 0) {
    console.log("无任何设备！");
    return;
  }
  let device = devices[0];

  let port = 27183;
  //   let scid = (
  //     "00000000" + Math.floor(Math.random() * 100000).toString(16)
  //   ).slice(-8);
  //   console.log("scid", scid);
  let scid = "00002478";
  //   await reverseServerPort(device.id, scid, port);
  await openSocketServer(port);
}

async function test2() {
  await closeSocketServer();
}

let unlisten: UnlistenFn | undefined;
onMounted(async () => {
  unlisten = await listen("device-reply", (event) => {
    console.log("device-reply:");
    console.log(event);
  });
});

onUnmounted(() => {
  if (unlisten !== undefined) unlisten();
});
</script>

<template>
  <div class="setting-page">
    <n-button @click="test">测试1</n-button>
    <n-button @click="test2">测试2</n-button>
  </div>
</template>

<style scoped></style>
