import { createI18n } from "vue-i18n";
import { Store } from "@tauri-apps/plugin-store";

import en from "./en.json";
import zh from "./zh.json";

const localStore = new Store("store.bin");
const language = (await localStore.get<string>("language")) ?? "en";

const i18n = createI18n({
  locale: language,
  allowComposition: true,
  messages: {
    en,
    zh,
  },
});

export default i18n;
