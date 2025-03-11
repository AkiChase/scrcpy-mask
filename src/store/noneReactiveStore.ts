import { LocalStore } from "./localStore";

interface MemType {
  screenStreamClientId: string;
  keyInputFlag: boolean;
}

interface LocalType {
  keySettingPos: { x: number; y: number };
  maskArea: {
    posX: number;
    posY: number;
    sizeW: number;
    sizeH: number;
  };
}

export class NonReactiveStore {
  static mem: MemType = {
    screenStreamClientId: "",
    keyInputFlag: false,
  };

  static local: LocalType = {
    keySettingPos: { x: 100, y: 100 },
    maskArea: { posX: 0, posY: 0, sizeW: 0, sizeH: 0 },
  };

  // local setter
  static async setLocal<K extends keyof LocalType>(
    key: K,
    value: LocalType[K]
  ) {
    await LocalStore.set(key, value);
    NonReactiveStore.local[key] = value;
  }
}
