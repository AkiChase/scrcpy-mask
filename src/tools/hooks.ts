import { getVersion } from "@tauri-apps/api/app";
import { useDialog, useMessage } from "naive-ui";
import { h } from "vue";
import { useI18n } from "vue-i18n";
import { fetch } from "@tauri-apps/plugin-http";
import { compareVersion } from "./tools";
import { checkAdbAvailable } from "./invoke";
import { NonReactiveStore } from "../store/noneReactiveStore";

// TODO use markdown to render update info

function renderUpdateInfo(content: string) {
  const pList = content.split("\r\n").map((line: string) => h("p", line));
  return h("div", { style: "margin: 20px 0" }, pList);
}

export function useCheckUpdate() {
  const message = useMessage();
  const dialog = useDialog();
  const { t } = useI18n();
  return async () => {
    try {
      const curVersion = await getVersion();
      const res = await fetch(
        "https://api.github.com/repos/AkiChase/scrcpy-mask/releases/latest",
        {
          connectTimeout: 5000,
        }
      );
      if (res.status !== 200) {
        message.error(t("pages.Mask.checkUpdate.failed"));
      } else {
        const data = await res.json();
        const latestVersion = (data.tag_name as string).slice(1);
        if (compareVersion(curVersion, latestVersion) >= 0) {
          message.success(
            t("pages.Mask.checkUpdate.isLatest", [latestVersion, curVersion])
          );
          return;
        }
        const body = data.body as string;
        dialog.info({
          title: t("pages.Mask.checkUpdate.notLatest.title", [latestVersion]),
          content: () => renderUpdateInfo(body),
          positiveText: t("pages.Mask.checkUpdate.notLatest.positiveText"),
          negativeText: t("pages.Mask.checkUpdate.notLatest.negativeText"),
          onPositiveClick: () => {
            open(data.html_url);
          },
        });
      }
    } catch (e) {
      console.error(e);
      message.error(t("pages.Mask.checkUpdate.failed"));
    }
  };
}

export function useCheckAdb() {
  const message = useMessage();
  const { t } = useI18n();

  return async function checkAdb() {
    try {
      if (NonReactiveStore.mem.adbUnavailableMsgIns) {
        NonReactiveStore.mem.adbUnavailableMsgIns.destroy();
        NonReactiveStore.mem.adbUnavailableMsgIns = null;
      }
      await checkAdbAvailable();
    } catch (e) {
      NonReactiveStore.mem.adbUnavailableMsgIns = message.error(
        t("pages.Mask.checkAdb", [e]),
        {
          duration: 0,
        }
      );
    }
  };
}
