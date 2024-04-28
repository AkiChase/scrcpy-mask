<script setup lang="ts">
import { computed, ref } from "vue";
import { useGlobalStore } from "../../store/global";
import { KeySteeringWheel } from "../../keyMappingConfig";

const emit = defineEmits<{
  edit: [];
}>();

const props = defineProps<{
  index: number;
}>();

const activeSteeringWheelButtonKeyIndex = defineModel(
  "activeSteeringWheelButtonKeyIndex",
  { required: true }
);

const activeIndex = defineModel("activeIndex", { required: true });

const store = useGlobalStore();
const elementRef = ref<HTMLElement | null>(null);

const offset = computed(() => {
  const keyboardElement = document.getElementById(
    "keyboardElement"
  ) as HTMLElement;
  const clientWidth = keyboardElement.clientWidth;
  const screenSizeW = store.screenSizeW === 0 ? clientWidth : store.screenSizeW;
  return (
    ((store.editKeyMappingList[props.index] as KeySteeringWheel).offset *
      clientWidth) /
    screenSizeW
  );
});

function dragHandler(downEvent: MouseEvent) {
  activeIndex.value = props.index;
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
      left: `${store.editKeyMappingList[props.index].posX - offset}px`,
      top: `${store.editKeyMappingList[props.index].posY - offset}px`,
      width: `${offset * 2}px`,
      height: `${offset * 2}px`,
    }"
    @mousedown="dragHandler"
    class="key-steering-wheel"
    ref="elementRef"
  >
    <i />
    <span
      @mousedown="activeSteeringWheelButtonKeyIndex = 0"
      :class="{
        'active-wheel':
          props.index === activeIndex && activeSteeringWheelButtonKeyIndex == 0,
      }"
      >{{
        (store.editKeyMappingList[props.index] as KeySteeringWheel).key.up
      }}</span
    >
    <i />
    <span
      @mousedown="activeSteeringWheelButtonKeyIndex = 2"
      :class="{
        'active-wheel':
          props.index === activeIndex && activeSteeringWheelButtonKeyIndex == 2,
      }"
      >{{
        (store.editKeyMappingList[props.index] as KeySteeringWheel).key.left
      }}</span
    >
    <i />
    <span
      @mousedown="activeSteeringWheelButtonKeyIndex = 3"
      :class="{
        'active-wheel':
          props.index === activeIndex && activeSteeringWheelButtonKeyIndex == 3,
      }"
      >{{
        (store.editKeyMappingList[props.index] as KeySteeringWheel).key.right
      }}</span
    >
    <i />
    <span
      @mousedown="activeSteeringWheelButtonKeyIndex = 1"
      :class="{
        'active-wheel':
          props.index === activeIndex && activeSteeringWheelButtonKeyIndex == 1,
      }"
      >{{
        (store.editKeyMappingList[props.index] as KeySteeringWheel).key.down
      }}</span
    >
    <i />
  </div>
</template>

<style scoped lang="scss">
.key-steering-wheel {
  position: absolute;
  border-radius: 50%;
  border: 2px solid var(--blue-color);
  display: flex;
  justify-content: center;
  align-items: center;
  font-size: 10px;
  font-weight: bold;

  display: grid;
  grid-template-columns: repeat(3, 33%);
  grid-template-rows: repeat(3, 33%);
  justify-items: center;
  align-items: center;

  &:not(.active):hover {
    border: 2px solid var(--light-color);
  }

  span {
    cursor: pointer;
    &:hover {
      color: var(--primary-hover-color);
    }
  }
}
.active {
  border: 2px solid var(--primary-color);
  z-index: 2;
}

.active-wheel {
  color: var(--primary-color);
}
</style>
