import { load, Store } from "@tauri-apps/plugin-store";
import { KeyMappingConfig } from "../keyMappingConfig";
import { useGlobalStore } from "./global";
import { setAdbPath } from "../invoke";
import { allLanguage } from "../i18n";
import { locale } from "@tauri-apps/plugin-os";
import { NonReactiveStore } from "./noneReactiveStore";
import {
  getCurrentWindow,
  LogicalPosition,
  LogicalSize,
  primaryMonitor,
} from "@tauri-apps/api/window";

export class LocalStore {
  public static store: Store;
  public static vueStore: ReturnType<typeof useGlobalStore>;

  static async init() {
    this.store = await load("store.json", { autoSave: true });
    this.vueStore = useGlobalStore();

    await initAdbPath();
    await initMaskArea();
    await initKeyMappingConfigList();
    await initCurKeyMappingIndex();
    await initLanugage();

    await this._load("maskKeyTip", {
      show: true,
      transparency: 0.5,
    });
    await this._load("checkUpdateAtStart", true);
    await this._load("rotation", {
      enable: true,
      verticalLength: 600,
      horizontalLength: 800,
    });
    await this._load("screenStream", {
      enable: false,
      address: "",
    });
    await this._load("clipboardSync", {
      syncFromDevice: true,
      pasteFromPC: true,
    });
    await this._load("keySettingPos", { x: 100, y: 100 });
  }

  static async _load(key: string, defaultValue: any) {
    const value = (await this.get(key)) ?? defaultValue;

    if (key in this.vueStore.$state) {
      this.vueStore.$patch({ [key]: value });
    } else if (key in NonReactiveStore.local) {
      NonReactiveStore.local[key as keyof typeof NonReactiveStore.local] =
        value;
    }
  }

  static async get<T>(key: string): Promise<T | undefined> {
    return this.store.get(key);
  }

  static async set<T>(key: string, value: T) {
    return this.store.set(key, value);
  }

  static async delete(key: string) {
    return this.store.delete(key);
  }

  static async clear() {
    return this.store.clear();
  }

  static async entries() {
    return this.store.entries();
  }
}

// init adbPath
async function initAdbPath() {
  const adbPath = (await LocalStore.get<string>("adbPath")) ?? "adb";
  await setAdbPath(adbPath);
  LocalStore.vueStore.adbPath = adbPath;
}

async function initMaskArea() {
  const maskArea = (await LocalStore.get<{
    posX: number;
    posY: number;
    sizeW: number;
    sizeH: number;
  }>("maskArea")) ?? { posX: 100, posY: 100, sizeW: 800, sizeH: 600 };

  // mask area validation
  const appWindow = getCurrentWindow();
  let { posX, posY, sizeW, sizeH } = maskArea;
  const mt = 30;
  const ml = 70;

  // min size
  if (sizeW < 120) sizeW = 120;
  if (sizeH < 150) sizeH = 120;
  // max size
  const monitor = await primaryMonitor();
  const monitorSize = monitor?.size.toLogical(monitor.scaleFactor);
  if (monitorSize !== undefined) {
    if (sizeW > monitorSize.width - ml) sizeW = monitorSize.width - ml;
    if (sizeH > monitorSize.height - mt) sizeH = monitorSize.height - mt;
  }
  [sizeW, sizeH] = [sizeW, sizeH].map((v) => Math.round(v));
  appWindow.setSize(new LogicalSize(sizeW + ml, sizeH + mt));

  // min pos (right bottom corner)
  // move to left top corner
  if (posX + sizeW < 0) posX = ml;
  if (posY + sizeH < 0) posY = mt;
  if (monitorSize !== undefined) {
    // max pos (left top corner)
    // move to right bottom corner
    if (posX > monitorSize.width) posX = monitorSize.width - sizeW;
    if (posY > monitorSize.height) posY = monitorSize.height - sizeH;
  }

  [posX, posY] = [posX, posY].map((v) => Math.round(v));
  appWindow.setPosition(new LogicalPosition(posX - 70, posY - 30));

  NonReactiveStore.setLocal("maskArea", {
    posX,
    posY,
    sizeW,
    sizeH,
  });
}

// init keyMappingConfigList from local store
async function initKeyMappingConfigList() {
  let keyMappingConfigList = await LocalStore.get<KeyMappingConfig[]>(
    "keyMappingConfigList"
  );

  if (keyMappingConfigList === undefined || keyMappingConfigList.length === 0) {
    // add empty key mapping config
    const maskArea = NonReactiveStore.local.maskArea;
    keyMappingConfigList = [
      {
        relativeSize: { w: maskArea.sizeW, h: maskArea.sizeH },
        title: "Default",
        list: [],
      },
    ];

    await LocalStore.set("keyMappingConfigList", keyMappingConfigList);
  }
  LocalStore.vueStore.keyMappingConfigList = keyMappingConfigList;
}

// init curKeyMappingIndex from local store
async function initCurKeyMappingIndex() {
  let curKeyMappingIndex = await LocalStore.get<number>("curKeyMappingIndex");
  if (
    curKeyMappingIndex === undefined ||
    curKeyMappingIndex >= LocalStore.vueStore.keyMappingConfigList.length
  ) {
    curKeyMappingIndex = 0;
    LocalStore.set("curKeyMappingIndex", curKeyMappingIndex);
  }
  LocalStore.vueStore.curKeyMappingIndex = curKeyMappingIndex;
}

async function initLanugage() {
  let language = await LocalStore.get<keyof typeof allLanguage>("language");
  if (language === undefined) {
    const lang = await locale();
    if (lang === null) language = "en-US";
    else if (lang in allLanguage) {
      language = lang as keyof typeof allLanguage;
    } else {
      if (lang.startsWith("zh")) language = "zh-CN";
      else if (lang.startsWith("en")) language = "en-US";
      else language = "en-US";
    }
  }

  LocalStore.vueStore.setLanguage(language);
}
