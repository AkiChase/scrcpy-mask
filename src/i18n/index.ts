import { createI18n } from "vue-i18n";
import { Store } from "@tauri-apps/plugin-store";

import enUS from "./en-US.json";
import zhCN from "./zh-CN.json";

const localStore = new Store("store.bin");
const language = (await localStore.get<string>("language")) ?? "en-US";

const i18n = createI18n({
  locale: language,
  allowComposition: true,
  messages: {
    "en-US": enUS,
    "zh-CN": zhCN,
  },
});

export default i18n;
