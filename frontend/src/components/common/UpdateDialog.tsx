import { Modal } from "antd";
import { useAppDispatch, useAppSelector } from "../../store/store";
import { setShowUpdateDialog } from "../../store/other";
import Markdown from "react-markdown";
import { useTranslation } from "react-i18next";
export default function UpdateDialog() {
  const updateInfo = useAppSelector((state) => state.other.updateInfo);
  const showUpdateDialog = useAppSelector(
    (state) => state.other.showUpdateDialog
  );
  const dispatch = useAppDispatch();
  const { t } = useTranslation();

  return (
    <Modal
      title={updateInfo.title}
      open={showUpdateDialog}
      okText={t("settings.about.download")}
      cancelText={t("settings.about.cancel")}
      onOk={() => {
        window.open("https://github.com/AkiChase/scrcpy-mask/releases/latest");
        dispatch(setShowUpdateDialog(false));
      }}
      onCancel={() => {
        dispatch(setShowUpdateDialog(false));
      }}
    >
      <div className="max-h-70vh overflow-y-auto">
        {new Date(updateInfo.time).toLocaleString()}
        <Markdown>{updateInfo.body}</Markdown>
      </div>
    </Modal>
  );
}
