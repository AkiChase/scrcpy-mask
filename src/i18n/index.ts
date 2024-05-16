import { createI18n } from "vue-i18n";
import { Store } from "@tauri-apps/plugin-store";

import enUS from "./en-US.json";
import zhCN from "./zh-CN.json";

const localStore = new Store("store.bin");

const i18n = createI18n({
  allowComposition: true,
  legacy: false,
  messages: {
    "en-US": enUS,
    "zh-CN": zhCN,
  },
});

localStore.get<"en-US" | "zh-CN">("language").then((language) => {
  i18n.global.locale.value = language ?? "en-US";
});

export default i18n;
