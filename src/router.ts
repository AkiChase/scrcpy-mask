import { createRouter, createWebHashHistory } from "vue-router";
import Mask from "./components/Mask.vue";
import Setting from "./components/setting/Setting.vue";
import KeyBoard from "./components/keyboard/KeyBoard.vue";
import Device from "./components/Device.vue";

const routes = [
  { path: "/", name: "mask", component: Mask },
  { path: "/device", name: "device", component: Device },
  { path: "/setting", name: "setting", component: Setting },
  { path: "/keyboard", name: "keyboard", component: KeyBoard },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

export default router;
