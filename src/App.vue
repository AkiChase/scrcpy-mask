<script setup lang="ts">
import Sidebar from "./components/sidebar/Sidebar.vue";
import Header from "./components/Header.vue";
import {
  darkTheme,
  NConfigProvider,
  NMessageProvider,
  NDialogProvider,
  NSpin,
  NFlex,
} from "naive-ui";
import { onMounted } from "vue";
import { useRouter } from "vue-router";

onMounted(() => {
  const router = useRouter();
  router.replace({ name: "mask" });
});
</script>

<template>
  <NConfigProvider :theme="darkTheme" class="root">
    <NMessageProvider>
      <NDialogProvider>
        <Header />
        <RouterView v-slot="{ Component }">
          <template v-if="Component">
            <KeepAlive>
              <Suspense>
                <component :is="Component" />
                <template #fallback>
                  <NFlex
                    justify="center"
                    align="center"
                    style="
                      grid-area: content;
                      background-color: var(--content-bg-color);
                    "
                  >
                    <NSpin :size="75" />
                  </NFlex>
                </template>
              </Suspense>
            </KeepAlive>
          </template>
        </RouterView>
        <Sidebar />
      </NDialogProvider>
    </NMessageProvider>
  </NConfigProvider>
</template>

<style>
.root {
  background-color: transparent;
  height: 100%;
  display: grid;
  grid-template-columns: 70px 1fr;
  grid-template-rows: 30px 1fr;
  grid-template-areas:
    "sidebar header"
    "sidebar content";
}

.n-message {
  user-select: none;
  -webkit-user-select: none;
}
</style>
