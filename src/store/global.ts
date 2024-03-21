import { defineStore } from "pinia";
import { ref } from "vue";

export const useGlobalStore = defineStore("counter", () => {
  const showLoadingRef = ref(false);
  function showLoading() {
    showLoadingRef.value = true;
  }
  function hideLoading() {
    showLoadingRef.value = false;
  }

  const isServerRunning = ref(false);

  return { showLoading, hideLoading, showLoadingRef, isServerRunning};
});
