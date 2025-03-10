import { createI18n } from "vue-i18n";
import { load } from "@tauri-apps/plugin-store";
import { locale } from "@tauri-apps/plugin-os";

import enUS from "./en-US.json";
import zhCN from "./zh-CN.json";
const localStore = await load("store.json");

export const allLanguage = {
  "en-US": { label: "English US", value: enUS },
  "zh-CN": { label: "简体中文", value: zhCN },
};

const i18n = createI18n({
  allowComposition: true,
  messages: Object.fromEntries(
    Object.entries(allLanguage).map(([key, value]) => [key, value.value])
  ),
});

localStore.get<"en-US" | "zh-CN">("language").then((language) => {
  if (language === undefined) {
    locale().then((lang) => {
      if (lang === null) i18n.global.locale = "en-US";
      else if (lang in allLanguage) {
        i18n.global.locale = lang;
      } else {
        if (lang.startsWith("zh")) i18n.global.locale = "zh-CN";
        else if (lang.startsWith("en")) i18n.global.locale = "en-US";
        else i18n.global.locale = "en-US";
      }
    });
    // "en-US"
  } else {
    i18n.global.locale = language;
  }
});

export default i18n;
