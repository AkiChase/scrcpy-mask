import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import enUS from "./en-US/translation.json";
import zhCN from "./zh-CN/translation.json";

i18n.use(initReactI18next).init({
  fallbackLng: "en-US",
  resources: {
    "en-US": { translation: enUS },
    "zh-CN": { translation: zhCN },
  },
  interpolation: {
    escapeValue: false, // not needed for react as it escapes by default
  },
});

export default i18n;
