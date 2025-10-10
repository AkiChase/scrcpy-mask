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
  BorderOutlined,
  BulbFilled,
  BulbOutlined,
  DisconnectOutlined,
  DownOutlined,
  EnterOutlined,
  InfoCircleOutlined,
  LinkOutlined,
  SwitcherOutlined,
  SyncOutlined,
  UnorderedListOutlined,
  UpOutlined,
} from "@ant-design/icons";
import IconButton from "./common/IconButton";
import { useEffect, useState } from "react";
import { ItemBox, ItemBoxContainer } from "./common/ItemBox";
import { setControlledDevices, setIsLoading } from "../store/other";
import { useMessageContext } from "../hooks";
import { useAppDispatch, useAppSelector } from "../store/store";
import { useLocation } from "react-router-dom";

function ControlledDevices({ refresh }: { refresh: () => void }) {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();
  const messageApi = useMessageContext();
  const controlledDevices = useAppSelector(
    (state) => state.other.controlledDevices
  );
  const [actionMenuOpen, setActionMenuOpen] = useState(false);

  const handleMenuOpenChange: DropdownProps["onOpenChange"] = (
    nextOpen,
    info
  ) => {
    if (info.source === "trigger" || nextOpen) {
      setActionMenuOpen(nextOpen);
    }
  };

  async function decontrolDevice(device_id: string) {
    dispatch(setIsLoading(true));
    try {
      const res = await requestPost("/api/device/decontrol_device", {
        device_id,
      });
      messageApi?.success(res.message);
      setTimeout(refresh, 1000);
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
                switch (key) {
                  case "SetDisplayPowerOff":
                    try {
                      await requestPost(
                        "/api/device/control/set_display_power",
                        {
                          mode: false,
                        }
                      );
                    } catch (error) {
                      messageApi?.error(error as string);
                    }
                    break;
                  case "SetDisplayPowerOn":
                    try {
                      await requestPost(
                        "/api/device/control/set_display_power",
                        {
                          mode: true,
                        }
                      );
                    } catch (error) {
                      messageApi?.error(error as string);
                    }
                    break;
                  case "VolumeUp":
                    try {
                      await requestPost("/api/device/control/send_key", {
                        keycode: "VolumeUp",
                      });
                    } catch (error) {
                      messageApi?.error(error as string);
                    }
                    break;
                  case "VolumeDown":
                    try {
                      await requestPost("/api/device/control/send_key", {
                        keycode: "VolumeDown",
                      });
                    } catch (error) {
                      messageApi?.error(error as string);
                    }
                    break;
                  case "Back":
                    try {
                      await requestPost("/api/device/control/send_key", {
                        keycode: "Back",
                      });
                    } catch (error) {
                      messageApi?.error(error as string);
                    }
                    break;
                  case "Home":
                    try {
                      await requestPost("/api/device/control/send_key", {
                        keycode: "Home",
                      });
                    } catch (error) {
                      messageApi?.error(error as string);
                    }
                    break;
                  case "AppSwitch":
                    try {
                      await requestPost("/api/device/control/send_key", {
                        keycode: "AppSwitch",
                      });
                    } catch (error) {
                      messageApi?.error(error as string);
                    }
                    break;
                  default:
                    break;
                }
              },
              items: [
                {
                  key: "SetDisplayPowerOff",
                  icon: <BulbOutlined />,
                  label: t("devices.actions.setDisplayPowerOff"),
                },
                {
                  key: "SetDisplayPowerOn",
                  icon: <BulbFilled />,
                  label: t("devices.actions.setDisplayPowerOn"),
                },
                {
                  key: "VolumeUp",
                  icon: <UpOutlined />,
                  label: t("devices.actions.volumeUp"),
                },
                {
                  key: "VolumeDown",
                  icon: <DownOutlined />,
                  label: t("devices.actions.volumeDown"),
                },
                {
                  key: "Back",
                  icon: <EnterOutlined />,
                  label: t("devices.actions.back"),
                },
                {
                  key: "Home",
                  icon: <BorderOutlined />,
                  label: t("devices.actions.home"),
                },
                {
                  key: "AppSwitch",
                  icon: <SwitcherOutlined />,
                  label: t("devices.actions.appSwitch"),
                },
              ],
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
  refresh,
}: {
  otherDevices: AdbDevice[];
  refresh: () => void;
}) {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();
  const messageApi = useMessageContext();

  const [isVideo, setIsVideo] = useState(false);
  const [displayID, setDisplayID] = useState(0);

  async function controlDevice(device: AdbDevice) {
    dispatch(setIsLoading(true));
    try {
      const res = await requestPost("/api/device/control_device", {
        device_id: device.id,
        display_id: displayID,
        video: isVideo,
      });
      messageApi?.success(res.message);
      setTimeout(refresh, 1000);
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

  const [connectAddr, setConnectAddr] = useState("");
  const [pairAddr, setPairAddr] = useState("");
  const [pairCode, setPairCode] = useState("");

  const [otherDevices, setOtherDevices] = useState<AdbDevice[]>([]);

  useEffect(() => {
    if (location.pathname === "/devices") refreshDevices();
  }, [location.pathname]);

  async function refreshDevices() {
    dispatch(setIsLoading(true));
    try {
      const res = await requestGet<{
        controlled_devices: ControlledDevice[];
        adb_devices: AdbDevice[];
      }>("/api/device/device_list");
      dispatch(setControlledDevices(res.data.controlled_devices));
      const controlled_id_set = new Set(
        res.data.controlled_devices.map((device) => device.device_id)
      );
      setOtherDevices(
        res.data.adb_devices.filter(
          (device) => !(device.id in controlled_id_set)
        )
      );
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
    dispatch(setIsLoading(true));
    try {
      const res = await requestPost("/api/device/adb_connect", {
        address: connectAddr,
      });
      messageApi?.success(res.message);
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
                onChange={(e) => setConnectAddr(e.target.value)}
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
        <ControlledDevices refresh={refreshDevices} />
      </section>
      <section className="mt-4">
        <h2 className="title-with-line">{t("devices.otherDevices.title")}</h2>
        <OtherDevices otherDevices={otherDevices} refresh={refreshDevices} />
      </section>
    </div>
  );
}
