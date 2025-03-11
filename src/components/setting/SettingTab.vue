<template>
  <div class="setting-tab">
    <div>
      <slot />
    </div>
  </div>
</template>

<style lang="scss" scoped>
@use "../../css/common.scss";

.setting-tab {
  display: flex;
  width: 100%;
  height: 100%;
  flex-direction: column;

  & > div {
    padding: 10px 25px;
    flex-grow: 1;
    height: 0;
    overflow-x: hidden;
    overflow-y: auto;
    @include common.scrollbar;
  }
}
</style>
