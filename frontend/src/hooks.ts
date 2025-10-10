import type { MessageInstance } from "antd/es/message/interface";
import { createContext, useContext } from "react";
import { useAppDispatch, useAppSelector } from "./store/store";
import { setBackgroundImage, setIsLoading } from "./store/other";
import { useTranslation } from "react-i18next";
import { requestPost } from "./utils";

export const MessageContext = createContext<MessageInstance | null>(null);
export const useMessageContext = () => useContext(MessageContext);

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
