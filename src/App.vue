<script setup lang="ts">
import Sidebar from "./components/Sidebar.vue";
import Header from "./components/Header.vue";
import {
  darkTheme,
  NConfigProvider,
  NMessageProvider,
  NDialogProvider,
} from "naive-ui";
</script>

<template>
  <NConfigProvider :theme="darkTheme" class="container">
    <NMessageProvider>
      <Header />
      <NDialogProvider>
          <RouterView v-slot="{ Component }">
            <KeepAlive>
              <component :is="Component" />
            </KeepAlive>
          </RouterView>
      </NDialogProvider>
      <Sidebar />
    </NMessageProvider>
  </NConfigProvider>
</template>

<style>
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
