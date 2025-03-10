import { createI18n } from "vue-i18n";

import enUS from "./en-US.json";
import zhCN from "./zh-CN.json";

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

export default i18n;
