import { useTranslation } from "react-i18next";
import { useAppDispatch, useAppSelector } from "../store/store";
import { ItemBox, ItemBoxContainer } from "./common/ItemBox";
import {
  Badge,
  Button,
  Flex,
  Input,
  InputNumber,
  Select,
  Slider,
  Space,
  Switch,
  Typography,
} from "antd";
import {
  forceSetLocalConfig,
  setAdbPath,
  setControllerPort,
  setVerticalPosition,
  setverticalMaskHeight,
  setWebPort,
  sethorizontalMaskWidth,
  setHorizontalPosition,
  setMappingLabelOpacity,
  setClipboardSync,
  setLanguage,
  setVideoCodec,
  setVideoBitRate,
  setVideoMaxSize,
  setVideoMaxFps,
  setAlwaysOnTop,
} from "../store/localConfig";
import {
  setIsLoading,
  setShowUpdateDialog,
  setUpdateInfo,
} from "../store/other";
import { requestGet } from "../utils";
import i18n from "../i18n";
import { useMessageContext } from "../hooks";
import {
  BilibiliFilled,
  CloudSyncOutlined,
  GithubFilled,
  InfoCircleOutlined,
  SyncOutlined,
} from "@ant-design/icons";

const languageOptions = [
  {
    label: "简体中文",
    value: "zh-CN",
  },
  {
    label: "English",
    value: "en-US",
  },
];

const videoCodecOptions = ["H264", "H265", "AV1"].map((v) => ({
  value: v,
  label: v,
}));

export default function Settings() {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();
  const messageApi = useMessageContext();
  const localConfig = useAppSelector((state) => state.localConfig);
  const updateInfo = useAppSelector((state) => state.other.updateInfo);

  async function loadLocalConfig() {
    dispatch(setIsLoading(true));
    try {
      const res = await requestGet("/api/config/get_config");
      dispatch(forceSetLocalConfig(res.data));
      i18n.changeLanguage(res.data.language);
    } catch (err: any) {
      messageApi?.error(err);
    }
    dispatch(setIsLoading(false));
  }

  async function openDataPath() {
    dispatch(setIsLoading(true));
    try {
      const res = await requestGet("/api/config/open_data_path");
      messageApi?.success(res.message);
    } catch (err: any) {
      messageApi?.error(err);
    }
    dispatch(setIsLoading(false));
  }

  async function checkUpdate() {
    try {
      const res = await requestGet("/api/config/check_update");
      dispatch(
        setUpdateInfo({
          currentVersion: res.data.current_version,
          hasUpdate: res.data.has_update,
          latestVersion: res.data.latest_version,
          title: res.data.title,
          body: res.data.body,
          time: res.data.time,
        })
      );
      if (res.data.has_update) {
        dispatch(setShowUpdateDialog(true));
      }
    } catch (err: any) {
      messageApi?.error(err);
    }
  }

  return (
    <div className="page-container">
      <section>
        <Flex align="start" justify="space-between">
          <h2 className="title-with-line" style={{ marginBottom: 0 }}>
            {t("settings.title.header")}
          </h2>
          <Button
            type="primary"
            icon={<SyncOutlined />}
            shape="circle"
            onClick={loadLocalConfig}
          />
        </Flex>
        <h3 className="title-with-line-sub">{t("settings.title.basic")}</h3>
        <ItemBoxContainer className="mb-6">
          <ItemBox label={t("settings.language")}>
            <Select
              className="w-sm"
              value={localConfig.language}
              options={languageOptions}
              onChange={(v) => dispatch(setLanguage(v))}
            />
          </ItemBox>
          <ItemBox label={t("settings.adbPath")}>
            <Input
              className="w-sm"
              value={localConfig.adbPath}
              onChange={(e) => dispatch(setAdbPath(e.target.value))}
            />
          </ItemBox>
          <ItemBox label={t("settings.clipboardSync")}>
            <Switch
              checked={localConfig.clipboardSync}
              onChange={(v) => dispatch(setClipboardSync(v))}
            />
          </ItemBox>
        </ItemBoxContainer>
        <h3 className="title-with-line-sub">{t("settings.title.mask")}</h3>
        <ItemBoxContainer className="mb-6">
          <ItemBox label={t("settings.alwaysOnTop")}>
            <Switch
              checked={localConfig.alwaysOnTop}
              onChange={(v) => dispatch(setAlwaysOnTop(v))}
            />
          </ItemBox>
          <ItemBox label={t("settings.mappingLabelOpacity")}>
            <Slider
              className="w-sm"
              min={0}
              max={1}
              step={0.01}
              onChange={(v) => dispatch(setMappingLabelOpacity(v))}
              value={localConfig.mappingLabelOpacity}
            />
          </ItemBox>
          <ItemBox label={t("settings.horizontalMaskWidth")}>
            <InputNumber
              className="w-sm"
              controls={false}
              min={50}
              value={localConfig.horizontalMaskWidth}
              onChange={(v) =>
                v !== null && dispatch(sethorizontalMaskWidth(v))
              }
            />
          </ItemBox>
          <ItemBox label={t("settings.horizontalMaskPosition")}>
            <Space.Compact className="w-sm">
              <InputNumber
                prefix="X:"
                className="w-50%"
                controls={false}
                value={localConfig.horizontalPosition[0]}
                onChange={(v) =>
                  v !== null &&
                  dispatch(
                    setHorizontalPosition([
                      v,
                      localConfig.horizontalPosition[1],
                    ])
                  )
                }
              />
              <InputNumber
                prefix="Y:"
                className="w-50%"
                controls={false}
                value={localConfig.horizontalPosition[1]}
                onChange={(v) =>
                  v !== null &&
                  dispatch(
                    setHorizontalPosition([
                      localConfig.horizontalPosition[0],
                      v,
                    ])
                  )
                }
              />
            </Space.Compact>
          </ItemBox>
          <ItemBox label={t("settings.verticalMaskHeight")}>
            <InputNumber
              className="w-sm"
              controls={false}
              min={50}
              value={localConfig.verticalMaskHeight}
              onChange={(v) => v !== null && dispatch(setverticalMaskHeight(v))}
            />
          </ItemBox>
          <ItemBox label={t("settings.verticalMaskPosition")}>
            <Space.Compact className="w-sm">
              <InputNumber
                prefix="X:"
                className="w-50%"
                controls={false}
                value={localConfig.verticalPosition[0]}
                onChange={(v) =>
                  v !== null &&
                  dispatch(
                    setVerticalPosition([v, localConfig.verticalPosition[1]])
                  )
                }
              />
              <InputNumber
                prefix="Y:"
                className="w-50%"
                controls={false}
                value={localConfig.verticalPosition[1]}
                onChange={(v) =>
                  v !== null &&
                  dispatch(
                    setVerticalPosition([localConfig.verticalPosition[0], v])
                  )
                }
              />
            </Space.Compact>
          </ItemBox>
        </ItemBoxContainer>
        <h3 className="title-with-line-sub">{t("settings.title.video")}</h3>
        <ItemBoxContainer className="mb-6">
          <ItemBox label={t("settings.videoCodec")}>
            <Select
              className="w-sm"
              value={localConfig.videoCodec}
              options={videoCodecOptions}
              onChange={(v) => dispatch(setVideoCodec(v))}
            />
          </ItemBox>
          <ItemBox label={t("settings.videoBitRate")}>
            <InputNumber
              className="w-sm"
              controls={false}
              min={1000000}
              suffix="bps"
              value={localConfig.videoBitRate}
              onChange={(v) => v !== null && dispatch(setVideoBitRate(v))}
            />
          </ItemBox>
          <ItemBox
            label={t("settings.videoMaxSize")}
            tooltip={t("settings.zeroUnlimitedTip")}
          >
            <InputNumber
              className="w-sm"
              controls={false}
              min={0}
              value={localConfig.videoMaxSize}
              onChange={(v) => v !== null && dispatch(setVideoMaxSize(v))}
            />
          </ItemBox>
          <ItemBox
            label={t("settings.videoMaxFps")}
            tooltip={t("settings.zeroUnlimitedTip")}
          >
            <InputNumber
              className="w-sm"
              controls={false}
              min={0}
              value={localConfig.videoMaxFps}
              onChange={(v) => v !== null && dispatch(setVideoMaxFps(v))}
            />
          </ItemBox>
        </ItemBoxContainer>

        <h3 className="title-with-line-sub">{t("settings.title.advance")}</h3>
        <ItemBoxContainer className="mb-6">
          <ItemBox label={t("settings.webPort")}>
            <InputNumber
              className="w-sm"
              controls={false}
              value={localConfig.webPort}
              onChange={(v) => v !== null && dispatch(setWebPort(v))}
            />
          </ItemBox>
          <ItemBox label={t("settings.controllerPort")}>
            <InputNumber
              className="w-sm"
              controls={false}
              value={localConfig.controllerPort}
              onChange={(v) => v !== null && dispatch(setControllerPort(v))}
            />
          </ItemBox>
          <ItemBox>
            <Button type="primary" onClick={openDataPath}>
              {t("settings.openDataPath")}
            </Button>
          </ItemBox>
        </ItemBoxContainer>
      </section>
      <section>
        <h2 className="title-with-line">{t("settings.about.title")}</h2>
        <Typography.Paragraph>{t("settings.about.intro")}</Typography.Paragraph>
        <Flex gap="large">
          <Button
            type="text"
            icon={<GithubFilled />}
            onClick={() =>
              window.open("https://github.com/AkiChase/scrcpy-mask", "_blank")
            }
          >
            Github
          </Button>
          <Button
            type="text"
            icon={<BilibiliFilled />}
            onClick={() =>
              window.open("https://space.bilibili.com/440760180", "_blank")
            }
          >
            BiliBili
          </Button>
        </Flex>
        <Flex gap="large" align="center" className="mt-4">
          <Button
            type="primary"
            icon={<CloudSyncOutlined />}
            onClick={checkUpdate}
          >
            {t("settings.about.checkUpdate")}
          </Button>
          <Badge dot={updateInfo.hasUpdate}>
            <Button
              type="primary"
              icon={<InfoCircleOutlined />}
              onClick={() => dispatch(setShowUpdateDialog(true))}
            >
              {t("settings.about.showUpdateDialog")}
            </Button>
          </Badge>
        </Flex>
        <Flex gap="large" align="center" className="mt-4">
          <Typography.Text>
            {t("settings.about.currentVersion")}: {updateInfo.currentVersion}
          </Typography.Text>
          <Typography.Text>
            {t("settings.about.latestVersion")}: {updateInfo.latestVersion}
          </Typography.Text>
        </Flex>
      </section>
    </div>
  );
}
