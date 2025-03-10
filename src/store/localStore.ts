import { load, Store } from "@tauri-apps/plugin-store";

export class LocalStore {
  public static store: Store;

  static async init() {
    this.store = await load("store.json");
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

  static async entries(){
    return this.store.entries();
  }
}
