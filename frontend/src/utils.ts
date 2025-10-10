import axios, { type AxiosResponse, type ResponseType } from "axios";

async function handleRequest<D = any>(
  req: () => Promise<AxiosResponse>
): Promise<{ message: string; data: D }> {
  try {
    const res = await req();
    if (res.headers["content-type"] === "application/json") {
      return { message: res.data.message, data: res.data.data };
    } else {
      return { message: "", data: res.data };
    }
  } catch (err: any) {
    let msg = "Request failed";
    if (err?.response) {
      const res = err.response as AxiosResponse;
      if (typeof res.data === "string") {
        if (err.response.data) {
          msg = err.response.data;
        } else {
          msg = `${res.statusText}: ${res.status}`;
        }
      } else {
        msg = err.response.data.message;
      }
    } else {
      msg = `Request failed: ${err}`;
    }
    throw msg;
  }
}

export async function requestGet<D = any>(
  url: string,
  params?: Record<string, string | number>,
  responseType?: ResponseType
): Promise<{ message: string; data: D }> {
  return await handleRequest(() => {
    const opt: any = { params };
    if (responseType) {
      opt.responseType = responseType;
    }
    return axios.get(url, opt);
  });
}

export async function requestPost<D = any>(
  url: string,
  data?: Record<string, any>,
  params?: Record<string, string | number>,
  responseType?: ResponseType
): Promise<{ message: string; data: D }> {
  return await handleRequest(() => {
    const opt: any = { params };
    if (responseType) {
      opt.responseType = responseType;
    }
    return axios.post(url, data, opt);
  });
}

export interface ControlledDevice {
  device_id: string;
  device_size: [number, number];
  main: boolean;
  name: string;
  scid: string;
  socket_ids: string[];
}

export interface AdbDevice {
  id: string;
  status: string;
}

export function deepClone<T>(value: T, cache = new WeakMap()): T {
  if (value === null || typeof value !== "object") {
    return value;
  }

  if (cache.has(value)) {
    return cache.get(value);
  }

  if (value instanceof Date) {
    return new Date(value.getTime()) as any;
  }

  if (value instanceof RegExp) {
    return new RegExp(value) as any;
  }

  if (value instanceof Map) {
    const result = new Map();
    cache.set(value, result);
    value.forEach((v, k) => {
      result.set(deepClone(k, cache), deepClone(v, cache));
    });
    return result as any;
  }

  if (value instanceof Set) {
    const result = new Set();
    cache.set(value, result);
    value.forEach((v) => {
      result.add(deepClone(v, cache));
    });
    return result as any;
  }

  if (Array.isArray(value)) {
    const result: any[] = [];
    cache.set(value, result);
    value.forEach((item, index) => {
      result[index] = deepClone(item, cache);
    });
    return result as any;
  }

  const result: Record<string | symbol, any> = {};
  cache.set(value, result);
  Reflect.ownKeys(value).forEach((key) => {
    result[key] = deepClone((value as any)[key], cache);
  });
  return result as T;
}

export function throttle<T extends (...args: any[]) => any>(
  func: T,
  delay: number
) {
  let lastCall = 0;
  let timeout: ReturnType<typeof setTimeout> | null = null;

  return function (...args: Parameters<T>) {
    const now = Date.now();

    const remaining = delay - (now - lastCall);

    if (remaining <= 0) {
      if (timeout) {
        clearTimeout(timeout);
        timeout = null;
      }
      lastCall = now;
      func(...args);
    } else if (!timeout) {
      timeout = setTimeout(() => {
        lastCall = Date.now();
        timeout = null;
        func(...args);
      }, remaining);
    }
  };
}

export function debounce<T extends (...args: any[]) => void>(
  fn: T,
  delay: number
): (...args: Parameters<T>) => void {
  let timer: ReturnType<typeof setTimeout> | null = null;

  return function (...args: Parameters<T>) {
    if (timer !== null) {
      clearTimeout(timer);
    }

    timer = setTimeout(() => {
      fn(...args);
    }, delay);
  };
}

export function toCamelCase(str: string): string {
  return str.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());
}
