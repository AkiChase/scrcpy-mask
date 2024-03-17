<script setup lang="ts">
import { NButton } from "naive-ui";
import {
  adbDevices,
  reverseServerPort,
  pushServerFile,
  openSocketServer,
  startScrcpyServer,
  getScreenSize,
  closeSocketServer,
} from "../../invoke";
import { onMounted, onUnmounted } from "vue";
import { UnlistenFn, listen } from "@tauri-apps/api/event";
import { sendInjectTouchEvent } from "../../frontcommand/controlMsg";
import {
  AndroidMotionEventAction,
  AndroidMotionEventButtons,
} from "../../frontcommand/android";

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

async function closeServer() {
  await closeSocketServer();
}

function sleep(time: number) {
  return new Promise<void>((resolve) => {
    setTimeout(() => {
      resolve();
    }, time);
  });
}

async function test3() {
  let devices = await adbDevices();
  if (devices.length == 0) {
    console.log("无任何设备！");
    return;
  }
  let device = devices[0];
  let size = await getScreenSize(device.id);

  const touch = async (x: number, y: number, up: boolean) => {
    await sendInjectTouchEvent({
      action: up
        ? AndroidMotionEventAction.AMOTION_EVENT_ACTION_UP
        : AndroidMotionEventAction.AMOTION_EVENT_ACTION_DOWN,
      actionButton: AndroidMotionEventButtons.AMOTION_EVENT_BUTTON_PRIMARY,
      buttons: AndroidMotionEventButtons.AMOTION_EVENT_BUTTON_PRIMARY,
      pointerId: 0,
      position: {
        x,
        y,
        w: size[0],
        h: size[1],
      },
      pressure: 1,
    });
  };
  for (let index = 0; index < 100; index++) {
    await touch(500, 1000 + 10 * index, false);
    await sleep(100);
  }
  await touch(500, 1300, true);
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
    <n-button @click="test3">测试3</n-button>
    <n-button @click="closeServer">关闭服务器</n-button>
  </div>
</template>

<style scoped></style>
