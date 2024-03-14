<script setup lang="ts">
import Sidebar from "./components/Sidebar.vue";
import Header from "./components/Header.vue";
import { darkTheme, NConfigProvider, NMessageProvider } from "naive-ui";


</script>

<template>
  <n-config-provider :theme="darkTheme" class="container">
    <n-message-provider>
      <Header />
      <router-view style="grid-area: content" v-slot="{ Component }">
        <keep-alive>
          <component :is="Component" />
        </keep-alive>
      </router-view>
      <Sidebar />
    </n-message-provider>
  </n-config-provider>
</template>

<style scoped>
.container {
  background-color: transparent;
  height: 100%;
  overflow: auto;
  display: grid;
  grid-template-columns: 70px 1fr;
  grid-template-rows: 30px 1fr;
  grid-template-areas:
    "sidebar header"
    "sidebar content";
}
</style>
