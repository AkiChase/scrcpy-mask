<script setup lang="ts">
import Sidebar from "./components/Sidebar.vue";
import Header from "./components/Header.vue";
import {
  darkTheme,
  NConfigProvider,
  NMessageProvider,
  NDialogProvider,
} from "naive-ui";
import { onMounted } from "vue";
import { useRouter } from "vue-router";

const router = useRouter();

onMounted(async () => {
  router.replace({ name: "mask" });
});
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
  display: grid;
  grid-template-columns: 70px 1fr;
  grid-template-rows: 30px 1fr;
  grid-template-areas:
    "sidebar header"
    "sidebar content";
}

.n-scrollbar-container {
  background-color: var(--bg-color);
}

.n-scrollbar-content {
  height: 100%;
}

.n-spin-content,
.n-spin-container {
  width: 100%;
  height: 100%;
}

.n-message {
  user-select: none;
  -webkit-user-select: none;
}
</style>
