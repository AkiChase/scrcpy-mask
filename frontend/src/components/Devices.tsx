import {
  Badge,
  Button,
  Checkbox,
  Descriptions,
  Dropdown,
  Flex,
  Input,
  InputNumber,
  Popover,
  Space,
  Table,
  Tag,
  type DropdownProps,
  type TableProps,
} from "antd";
import { useTranslation } from "react-i18next";
import {
  requestGet,
  requestPost,
  type AdbDevice,
  type ControlledDevice,
} from "../utils";
import {
  AimOutlined,
  BorderOutlined,
  BulbFilled,
  BulbOutlined,
  DisconnectOutlined,
  DownOutlined,
  EnterOutlined,
  InfoCircleOutlined,
  LinkOutlined,
  ReloadOutlined,
  SwitcherOutlined,
  SyncOutlined,
  UnorderedListOutlined,
  UpOutlined,
} from "@ant-design/icons";
import IconButton from "./common/IconButton";
import { useEffect, useMemo, useRef, useState } from "react";
import { ItemBox, ItemBoxContainer } from "./common/ItemBox";
import { setAdbDevices, setControlledDevices, setIsLoading } from "../store/other";
import { useMessageContext } from "../hooks";
import { useAppDispatch, useAppSelector } from "../store/store";
import { useLocation } from "react-router-dom";
import { setAdbConnectAddress } from "../store/localConfig";

function ControlledDevices({
  displayID,
  isVideo,
}: {
  displayID: number;
  isVideo: boolean;
}) {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();
  const messageApi = useMessageContext();
  const controlledDevices = useAppSelector(
    (state) => state.other.controlledDevices,
  );
  const deviceRotations = useAppSelector(
    (state) => state.other.deviceRotations,
  );
  const [actionMenuOpen, setActionMenuOpen] = useState(false);
  const actionMessageKey = "controlled-device-action";

  const handleMenuOpenChange: DropdownProps["onOpenChange"] = (
    nextOpen,
    info,
  ) => {
    if (info.source === "trigger" || nextOpen) {
      setActionMenuOpen(nextOpen);
    }
  };

  async function runDeviceAction(
    label: string,
    request: () => Promise<unknown>,
  ) {
    messageApi?.open({
      key: actionMessageKey,
      type: "loading",
      content: t("devices.actions.executing", { action: label }),
      duration: 0,
    });

    try {
      await request();
      messageApi?.open({
        key: actionMessageKey,
        type: "success",
        content: t("devices.actions.executed", { action: label }),
        duration: 2,
      });
    } catch (error) {
      messageApi?.open({
        key: actionMessageKey,
        type: "error",
        content: error as string,
        duration: 4,
      });
    }
  }

  const deviceActionItems = [
    {
      key: "SetDisplayPowerOff",
      icon: <BulbOutlined />,
      label: t("devices.actions.setDisplayPowerOff"),
      request: () =>
        requestPost("/api/device/control/set_display_power", {
          mode: false,
        }),
    },
    {
      key: "SetDisplayPowerOn",
      icon: <BulbFilled />,
      label: t("devices.actions.setDisplayPowerOn"),
      request: () =>
        requestPost("/api/device/control/set_display_power", {
          mode: true,
        }),
    },
    {
      key: "EnablePointerDebug",
      icon: <AimOutlined />,
      label: t("devices.actions.enablePointerDebug"),
      request: () =>
        requestPost("/api/device/control/set_pointer_location", {
          mode: true,
        }),
    },
    {
      key: "DisablePointerDebug",
      icon: <AimOutlined />,
      label: t("devices.actions.disablePointerDebug"),
      request: () =>
        requestPost("/api/device/control/set_pointer_location", {
          mode: false,
        }),
    },
    {
      key: "VolumeUp",
      icon: <UpOutlined />,
      label: t("devices.actions.volumeUp"),
      request: () =>
        requestPost("/api/device/control/send_key", {
          keycode: "VolumeUp",
        }),
    },
    {
      key: "VolumeDown",
      icon: <DownOutlined />,
      label: t("devices.actions.volumeDown"),
      request: () =>
        requestPost("/api/device/control/send_key", {
          keycode: "VolumeDown",
        }),
    },
    {
      key: "Back",
      icon: <EnterOutlined />,
      label: t("devices.actions.back"),
      request: () =>
        requestPost("/api/device/control/send_key", {
          keycode: "Back",
        }),
    },
    {
      key: "Home",
      icon: <BorderOutlined />,
      label: t("devices.actions.home"),
      request: () =>
        requestPost("/api/device/control/send_key", {
          keycode: "Home",
        }),
    },
    {
      key: "AppSwitch",
      icon: <SwitcherOutlined />,
      label: t("devices.actions.appSwitch"),
      request: () =>
        requestPost("/api/device/control/send_key", {
          keycode: "AppSwitch",
        }),
    },
  ];

  async function decontrolDevice(device_id: string) {
    dispatch(setIsLoading(true));
    try {
      const res = await requestPost("/api/device/decontrol_device", {
        device_id,
      });
      messageApi?.success(res.message);
    } catch (error) {
      messageApi?.error(error as string);
    }
    dispatch(setIsLoading(false));
  }

  async function reconnectDevice(device_id: string) {
    dispatch(setIsLoading(true));
    try {
      const res = await requestPost("/api/device/reconnect_device", {
        device_id,
        display_id: displayID,
        video: isVideo,
      });
      messageApi?.success(res.message);
    } catch (error) {
      messageApi?.error(error as string);
    }
    dispatch(setIsLoading(false));
  }

  const columns: TableProps<ControlledDevice>["columns"] = [
    {
      title: "ID",
      dataIndex: "device_id",
      key: "device_id",
      render: (_, record) => (
        <Space size="large">
          {record.device_id}
          {record.main && (
            <Badge
              color="green"
              text={t("devices.controlledDevices.mainDevice")}
            />
          )}
        </Space>
      ),
    },
    {
      title: t("devices.controlledDevices.name"),
      dataIndex: "name",
      key: "name",
    },
    {
      title: t("devices.controlledDevices.size"),
      dataIndex: "device_size",
      key: "device_size",
      render: (device_size) => {
        return `${device_size[0]}x${device_size[1]}`;
      },
    },
    {
      title: t("devices.controlledDevices.rotation"),
      key: "rotation",
      align: "center",
      render: (_, record) => {
        const rot = deviceRotations[record.scid];
        if (!rot) return null;
        const isLandscape = rot.width >= rot.height;
        return (
          <Tag color={isLandscape ? "green" : "blue"}>
            {isLandscape
              ? t("devices.controlledDevices.landscape")
              : t("devices.controlledDevices.portrait")}
          </Tag>
        );
      },
    },
    {
      title: (
        <Flex align="center" gap="middle" justify="center">
          <div>{t("devices.controlledDevices.action")}</div>
          <Dropdown
            trigger={["click"]}
            menu={{
              style: {
                userSelect: "none",
              },
              onClick: async ({ key }) => {
                const action = deviceActionItems.find(
                  (item) => item.key === key,
                );
                if (action) {
                  await runDeviceAction(action.label, action.request);
                }
              },
              items: deviceActionItems.map((item) => ({
                key: item.key,
                icon: item.icon,
                label: item.label,
              })),
            }}
            onOpenChange={handleMenuOpenChange}
            open={actionMenuOpen}
          >
            <div>
              <IconButton
                size={18}
                color="primary"
                icon={<UnorderedListOutlined />}
              />
            </div>
          </Dropdown>
        </Flex>
      ),
      key: "action",
      align: "center",
      render: (_, record) => (
        <Space size="middle" className="text-4">
          <Popover
            trigger="click"
            content={
              <Descriptions
                className="w-15rem"
                column={1}
                items={[
                  {
                    key: "scid",
                    label: "SCID",
                    children: record.scid,
                  },
                  {
                    key: "sockets",
                    label: "Sockets",
                    children: (
                      <Space direction="vertical" size={2}>
                        {record.socket_ids.map((socket_id) => (
                          <span key={socket_id}>{socket_id}</span>
                        ))}
                      </Space>
                    ),
                  },
                ]}
              />
            }
          >
            <IconButton
              tooltip={t("devices.controlledDevices.actionInfo")}
              size={18}
              color="info"
              icon={<InfoCircleOutlined />}
            />
          </Popover>
          <IconButton
            tooltip={t("devices.controlledDevices.actionReconnect")}
            size={18}
            color="warning"
            icon={<ReloadOutlined />}
            onClick={() => reconnectDevice(record.device_id)}
          />
          <IconButton
            tooltip={t("devices.controlledDevices.actionClose")}
            size={18}
            color="primary"
            icon={<DisconnectOutlined />}
            onClick={() => decontrolDevice(record.device_id)}
          />
        </Space>
      ),
    },
  ];

  return (
    <Table<ControlledDevice>
      rowKey={(record) => record.device_id}
      pagination={{ pageSize: 5 }}
      columns={columns}
      dataSource={controlledDevices}
    />
  );
}

function OtherDevices({
  otherDevices,
  videoState,
  displayIDState,
}: {
  otherDevices: AdbDevice[];
  videoState: [boolean, React.Dispatch<React.SetStateAction<boolean>>];
  displayIDState: [number, React.Dispatch<React.SetStateAction<number>>];
}) {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();
  const messageApi = useMessageContext();

  const [isVideo, setIsVideo] = videoState;
  const [displayID, setDisplayID] = displayIDState;

  async function controlDevice(device: AdbDevice) {
    dispatch(setIsLoading(true));
    try {
      const res = await requestPost("/api/device/control_device", {
        device_id: device.id,
        display_id: displayID,
        video: isVideo,
      });
      messageApi?.success(res.message);
    } catch (error) {
      messageApi?.error(error as string);
    }
    dispatch(setIsLoading(false));
  }

  const columns: TableProps<AdbDevice>["columns"] = [
    {
      title: "ID",
      dataIndex: "id",
      key: "id",
    },
    {
      title: t("devices.otherDevices.status"),
      dataIndex: "status",
      key: "status",
    },
    {
      title: (
        <Popover
          trigger="hover"
          content={
            <InputNumber
              className="w-full"
              value={displayID}
              onChange={(v) => v !== null && setDisplayID(v)}
            />
          }
          title="Display id"
        >
          <Flex justify="center" align="center">
            <Checkbox
              checked={isVideo}
              onChange={(e) => setIsVideo(e.target.checked)}
            >
              {t("devices.otherDevices.video")}
            </Checkbox>
          </Flex>
        </Popover>
      ),
      key: "action",
      align: "center",
      width: "18.5%",
      render: (_, record) => (
        <Space size="middle" className="text-4">
          <IconButton
            color="primary"
            tooltip={t("devices.otherDevices.actionControl")}
            size={18}
            icon={<LinkOutlined />}
            onClick={() => controlDevice(record)}
          />
        </Space>
      ),
    },
  ];

  return (
    <Table<AdbDevice>
      rowKey={(record) => record.id}
      pagination={{ pageSize: 5 }}
      columns={columns}
      dataSource={otherDevices}
    />
  );
}

export default function Devices() {
  const { t } = useTranslation();
  const messageApi = useMessageContext();
  const dispatch = useAppDispatch();
  const location = useLocation();

  const savedConnectAddr = useAppSelector(
    (state) => state.localConfig.adbConnectAddress,
  );
  const [connectAddr, setConnectAddr] = useState("");
  const [pairAddr, setPairAddr] = useState("");
  const [pairCode, setPairCode] = useState("");
  const connectAddrEditedRef = useRef(false);

  const controlledDevices = useAppSelector(
    (state) => state.other.controlledDevices,
  );
  const adbDevices = useAppSelector((state) => state.other.adbDevices);
  const otherDevices = useMemo(() => {
    const controlledIdSet = new Set(controlledDevices.map((d) => d.device_id));
    return adbDevices.filter((d) => !controlledIdSet.has(d.id));
  }, [controlledDevices, adbDevices]);

  const videoState = useState(false);
  const displayIDState = useState(0);

  useEffect(() => {
    if (location.pathname === "/devices") refreshDevices();
  }, [location.pathname]);

  useEffect(() => {
    if (!connectAddrEditedRef.current) {
      setConnectAddr(savedConnectAddr);
    }
  }, [savedConnectAddr]);

  function changeConnectAddr(value: string) {
    connectAddrEditedRef.current = true;
    setConnectAddr(value);
  }

  async function refreshDevices() {
    dispatch(setIsLoading(true));
    try {
      const res = await requestGet<{
        controlled_devices: ControlledDevice[];
        adb_devices: AdbDevice[];
      }>("/api/device/device_list");
      dispatch(setControlledDevices(res.data.controlled_devices));
      dispatch(setAdbDevices(res.data.adb_devices));
      messageApi?.success(res.message);
    } catch (error) {
      messageApi?.error(error as string);
    }
    dispatch(setIsLoading(false));
  }

  async function pairDevice() {
    dispatch(setIsLoading(true));
    try {
      const res = await requestPost("/api/device/adb_pair", {
        address: pairAddr,
        code: pairCode,
      });
      messageApi?.success(res.message);
      setTimeout(refreshDevices, 1000);
    } catch (error) {
      messageApi?.error(error as string);
    }
    dispatch(setIsLoading(false));
  }

  async function connectDevice() {
    const address = connectAddr.trim();
    dispatch(setIsLoading(true));
    try {
      const res = await requestPost("/api/device/adb_connect", {
        address,
      });
      messageApi?.success(res.message);
      connectAddrEditedRef.current = false;
      setConnectAddr(address);
      dispatch(setAdbConnectAddress(address));
      setTimeout(refreshDevices, 1000);
    } catch (error) {
      messageApi?.error(error as string);
    }
    dispatch(setIsLoading(false));
  }

  return (
    <div className="page-container">
      <section>
        <h2 className="title-with-line">{t("devices.adbTools.title")}</h2>
        <ItemBoxContainer className="mb-6">
          <ItemBox label={t("devices.adbTools.pair.label")}>
            <Space.Compact>
              <Input
                placeholder="ip:port"
                value={pairAddr}
                onChange={(e) => setPairAddr(e.target.value)}
              />
              <Input
                placeholder="code"
                value={pairCode}
                onChange={(e) => setPairCode(e.target.value)}
              />
              <Button type="primary" onClick={pairDevice}>
                {t("devices.adbTools.pair.btn")}
              </Button>
            </Space.Compact>
          </ItemBox>
          <ItemBox label={t("devices.adbTools.connect.label")}>
            <Space.Compact>
              <Input
                placeholder="ip:port"
                value={connectAddr}
                onChange={(e) => changeConnectAddr(e.target.value)}
              />
              <Button type="primary" onClick={connectDevice}>
                {t("devices.adbTools.connect.btn")}
              </Button>
            </Space.Compact>
          </ItemBox>
        </ItemBoxContainer>
      </section>
      <section>
        <Flex justify="space-between" align="start">
          <h2 className="title-with-line">
            {t("devices.controlledDevices.title")}
          </h2>
          <Button
            type="primary"
            icon={<SyncOutlined />}
            onClick={() => refreshDevices()}
          >
            {t("devices.common.refresh")}
          </Button>
        </Flex>
        <ControlledDevices
          displayID={displayIDState[0]}
          isVideo={videoState[0]}
        />
      </section>
      <section className="mt-4">
        <h2 className="title-with-line">{t("devices.otherDevices.title")}</h2>
        <OtherDevices
          otherDevices={otherDevices}
          videoState={videoState}
          displayIDState={displayIDState}
        />
      </section>
    </div>
  );
}
