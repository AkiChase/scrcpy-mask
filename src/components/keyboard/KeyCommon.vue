<script setup lang="ts">
import { ref } from "vue";
import { useGlobalStore } from "../../store/global";

const emit = defineEmits<{
  edit: [];
  active: [];
}>();

const props = defineProps<{
  index: number;
  activeIndex: number;
}>();
const store = useGlobalStore();
const elementRef = ref<HTMLElement | null>(null);

function dragHandler(downEvent: MouseEvent) {
  emit("active");
  const oldX = store.editKeyMappingList[props.index].posX;
  const oldY = store.editKeyMappingList[props.index].posY;
  const element = elementRef.value;
  if (element) {
    const keyboardElement = document.getElementById(
      "keyboardElement"
    ) as HTMLElement;
    const maxX = keyboardElement.clientWidth - 40;
    const maxY = keyboardElement.clientHeight - 40;

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
      left: `${store.editKeyMappingList[props.index].posX - 20}px`,
      top: `${store.editKeyMappingList[props.index].posY - 20}px`,
    }"
    @mousedown="dragHandler"
    class="key-common"
    ref="elementRef"
  >
    {{ store.editKeyMappingList[props.index].key }}
  </div>
</template>

<style scoped lang="scss">
.key-common {
  position: absolute;
  height: 40px;
  width: 40px;
  border-radius: 50%;
  border: 2px solid var(--blue-color);
  display: flex;
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
}
</style>
