<script setup lang="ts">
import { NButton } from "naive-ui";
import {
  adbDevices,
  reverseServerPort,
  pushServerFile,
  openSocketServer,
  startScrcpyServer,
} from "../../invoke";
import { onMounted, onUnmounted } from "vue";
import { UnlistenFn, listen } from "@tauri-apps/api/event";
import { shutdown } from "../../frontcommand/scrcpyMaskCmd";

let deviceName: string | undefined;
let scId: string | undefined;

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
  await reverseServerPort(device.id, scid, port);
  await openSocketServer(port);
}

async function test2() {
  let devices = await adbDevices();
  if (devices.length == 0) {
    console.log("无任何设备！");
    return;
  }
  let device = devices[0];
  let scid = "00002478";

  await pushServerFile(device.id);
  await startScrcpyServer(device.id, scid);
}


// function sleep(time: number) {
//   return new Promise<void>((resolve) => {
//     setTimeout(() => {
//       resolve();
//     }, time);
//   });
// }

async function test3() {
  let devices = await adbDevices();
  if (devices.length == 0) {
    console.log("无任何设备！");
    return;
  }
  await shutdown({ scId: scId || "" });
}

let unlisten: UnlistenFn | undefined;
onMounted(async () => {
  unlisten = await listen("device-reply", (event) => {
    try {
      let payload = JSON.parse(event.payload as string);
      switch (payload.msg) {
        case "MetaData":
          deviceName = payload.deviceName;
          scId = payload.scId;
          console.log("设备名", deviceName);
          console.log("scId", scId);
          break;
        case "ClipboardChanged":
          console.log("剪切板变动", payload.clipboard);
          break;
        case "ClipboardSetAck":
          console.log("剪切板设置成功", payload.sequence);
          break;
        default:
          console.log("Known reply", payload);
          break;
      }
    } catch (e) {
      console.error(e);
    }
  });
});

onUnmounted(() => {
  if (unlisten !== undefined) unlisten();
});
</script>

<template>
  <div class="setting-page">
    <NButton @click="test">测试1</NButton>
    <NButton @click="test2">测试2</NButton>
    <NButton @click="test3">测试3</NButton>
  </div>
</template>

<style scoped></style>
