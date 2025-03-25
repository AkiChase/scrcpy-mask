import { createApp } from "vue";
import { createPinia } from "pinia";
import "./css/global.scss";
import App from "./App.vue";
import router from "./router";
import i18n from "./i18n";
import { attachConsole } from "@tauri-apps/plugin-log";

attachConsole()

const pinia = createPinia();

const app = createApp(App);
app.use(router);
app.use(pinia);
app.use(i18n);
app.mount("#app");
