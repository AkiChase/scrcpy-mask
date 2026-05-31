import type { MessageInstance } from "antd/es/message/interface";
import { createContext, useCallback, useContext, useEffect, useRef } from "react";
import { useAppDispatch, useAppSelector } from "./store/store";
import { setAdbDevices, setBackgroundImage, setControlledDevices, setIsLoading } from "./store/other";
import { useTranslation } from "react-i18next";
import { requestGet, requestPost } from "./utils";
import { createFromIconfontCN } from "@ant-design/icons";

export const MessageContext = createContext<MessageInstance | null>(null);
export const useMessageContext = () => useContext(MessageContext);

export const IconFont = createFromIconfontCN({
  scriptUrl: new URL("./assets/iconfont.js", import.meta.url).href,
});

export function useRefreshBackgroundImage() {
  const dispatch = useAppDispatch();
  const messageApi = useMessageContext();
  const { t } = useTranslation();
  const controlledDevices = useAppSelector(
    (state) => state.other.controlledDevices
  );

  return async (silent: boolean = false) => {
    if (controlledDevices.length > 0) {
      const device = controlledDevices.find((d) => d.main === true);
      if (device) {
        if (!silent) dispatch(setIsLoading(true));
        try {
          const res = await requestPost(
            "/api/device/adb_screenshot",
            {
              id: device.device_id,
            },
            undefined,
            "blob"
          );

          if (res.data instanceof Blob) {
            const url = URL.createObjectURL(res.data);
            dispatch(setBackgroundImage(url));
            if (!silent)
              messageApi?.success(t("mappings.common.refreshSuccess"));
          } else {
            if (!silent)
              messageApi?.error(
                t("mappings.common.refreshBgFail", "unknown data type")
              );
          }
        } catch (error) {
          if (!silent)
            messageApi?.error(
              t("mappings.common.refreshBgFail", error as string)
            );
        }
        if (!silent) dispatch(setIsLoading(false));
        return;
      }
    }
    if (!silent) messageApi?.error(t("mappings.common.noMainDevice"));
  };
}

export function useDeviceWebSocket() {
  const dispatch = useAppDispatch();
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimerRef = useRef<number | null>(null);
  const mountedRef = useRef(true);

  const refresh = useCallback(async () => {
    try {
      const res = await requestGet<{
        controlled_devices: Array<{
          device_id: string;
          device_size: [number, number];
          main: boolean;
          name: string;
          scid: string;
          socket_ids: string[];
        }>;
        adb_devices: Array<{ id: string; status: string }>;
      }>("/api/device/device_list");
      dispatch(setControlledDevices(res.data.controlled_devices));
      dispatch(setAdbDevices(res.data.adb_devices));
    } catch {
      // silent refresh on ws trigger
    }
  }, [dispatch]);

  const connect = useCallback(() => {
    if (!mountedRef.current) return;

    const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    const ws = new WebSocket(`${protocol}//${window.location.host}/api/ws/connect`);

    ws.onmessage = () => {
      refresh();
    };

    ws.onclose = () => {
      if (mountedRef.current) {
        reconnectTimerRef.current = setTimeout(connect, 3000);
      }
    };

    ws.onerror = () => {
      ws.close();
    };

    wsRef.current = ws;
  }, [refresh]);

  useEffect(() => {
    mountedRef.current = true;
    connect();

    return () => {
      mountedRef.current = false;
      if (reconnectTimerRef.current !== null) {
        clearTimeout(reconnectTimerRef.current);
      }
      wsRef.current?.close();
    };
  }, [connect]);
}
