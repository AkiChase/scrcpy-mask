<script setup lang="ts">
import { ref } from "vue";
import { useGlobalStore } from "../../store/global";
import { Flash } from "@vicons/ionicons5";
import { NIcon } from "naive-ui";
const emit = defineEmits<{
  edit: [];
}>();

const props = defineProps<{
  index: number;
}>();

const activeIndex = defineModel("activeIndex", { required: true });

const store = useGlobalStore();
const elementRef = ref<HTMLElement | null>(null);

function dragHandler(downEvent: MouseEvent) {
  activeIndex.value = props.index;
  const oldX = store.editKeyMappingList[props.index].posX;
  const oldY = store.editKeyMappingList[props.index].posY;
  const element = elementRef.value;
  if (element) {
    const keyboardElement = document.getElementById(
      "keyboardElement"
    ) as HTMLElement;
    const maxX = keyboardElement.clientWidth - 60;
    const maxY = keyboardElement.clientHeight - 60;

    const x = downEvent.clientX;
    const y = downEvent.clientY;
    const moveHandler = (moveEvent: MouseEvent) => {
      let newX = oldX + moveEvent.clientX - x;
      let newY = oldY + moveEvent.clientY - y;
      newX = Math.max(0, Math.min(newX, maxX));
      newY = Math.max(0, Math.min(newY, maxY));
      store.editKeyMappingList[props.index].posX = newX;
      store.editKeyMappingList[props.index].posY = newY;
    };
    window.addEventListener("mousemove", moveHandler);
    const upHandler = () => {
      window.removeEventListener("mousemove", moveHandler);
      window.removeEventListener("mouseup", upHandler);
      if (
        oldX !== store.editKeyMappingList[props.index].posX ||
        oldY !== store.editKeyMappingList[props.index].posY
      ) {
        emit("edit");
      }
    };
    window.addEventListener("mouseup", upHandler);
  }
}
</script>

<template>
  <div
    :class="{ active: props.index === activeIndex }"
    :style="{
      left: `${store.editKeyMappingList[props.index].posX - 30}px`,
      top: `${store.editKeyMappingList[props.index].posY - 30}px`,
    }"
    @mousedown="dragHandler"
    class="key-skill"
    ref="elementRef"
  >
    <NIcon size="25"><Flash style="color: var(--blue-color)" /></NIcon>
    <span>{{ store.editKeyMappingList[props.index].key }}</span>
  </div>
</template>

<style scoped lang="scss">
.key-skill {
  position: absolute;
  height: 60px;
  width: 60px;
  border-radius: 50%;
  border: 2px solid var(--blue-color);
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  font-size: 10px;
  font-weight: bold;
  cursor: pointer;

  &:not(.active):hover {
    border: 2px solid var(--light-color);
  }
}
.active {
  border: 2px solid var(--primary-color);
  color: var(--primary-color);
  z-index: 2;
}
</style>
