import { load, Store } from "@tauri-apps/plugin-store";
import { KeyMappingConfig } from "../keyMappingConfig";
import { useGlobalStore } from "./global";
import { setAdbPath } from "../invoke";
import { allLanguage } from "../i18n";
import { locale } from "@tauri-apps/plugin-os";
import { NonReactiveStore } from "./noneReactiveStore";

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

  // TODO check mask area valid and remove related code in rust
  NonReactiveStore.setLocal("maskArea", maskArea);
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
