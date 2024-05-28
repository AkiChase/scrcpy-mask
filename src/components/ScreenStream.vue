<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from "vue";
import { useGlobalStore } from "../store/global";
import { MessageReactive, useMessage } from "naive-ui";
import { ScreenStream } from "../screenStream";

const props = defineProps<{
  cid: string;
}>();

const store = useGlobalStore();
const message = useMessage();

const streamImg = ref<HTMLImageElement | null>(null);

let msgReactive: MessageReactive | null = null;

function connectScreenStream() {
  if (streamImg.value) {
    const ss = new ScreenStream(streamImg.value, props.cid);
    ss.connect(
      store.screenStream.address,
      () => {},
      () => {
        msgReactive = message.error("投屏连接失败。关闭此信息将尝试重新连接", {
          duration: 0,
          closable: true,
          onClose: () => connectScreenStream(),
        });
      }
    );
  }
}

onMounted(() => {
  connectScreenStream();
});

onBeforeUnmount(() => {
  if (streamImg.value) streamImg.value.src = "";
  if (msgReactive) {
    msgReactive.destroy();
  }
});
</script>

<template>
  <div class="screen-stream">
    <img
      :style="{
        width: `${store.maskSizeW}px`,
        height: `${store.maskSizeH}px`,
      }"
      ref="streamImg"
    />
  </div>
</template>

<style scoped lang="scss">
.screen-stream {
  position: absolute;
  left: 70px;
  top: 30px;
  height: 100%;
  width: 100%;
  z-index: 0;
  pointer-events: none;
  user-select: none;
  -webkit-user-select: none;

  img {
    pointer-events: none;
    user-select: none;
    -webkit-user-select: none;
  }
}
</style>
