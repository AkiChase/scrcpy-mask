import { Flex, Menu, Layout } from "antd";
import { useState } from "react";
import logo from "../assets/128x128.png";
import { createFromIconfontCN, SettingFilled } from "@ant-design/icons";
import { useTranslation } from "react-i18next";
import { useLocation, useNavigate } from "react-router-dom";

const IconFont = createFromIconfontCN({
  scriptUrl: new URL("../assets/iconfont.js", import.meta.url).href,
});

export default function Sider() {
  const { t } = useTranslation();
  const location = useLocation();
  const navigate = useNavigate();
  const [siderCollapsed, setSiderCollapsed] = useState(true);

  const brandClass = siderCollapsed
    ? "opacity-0 max-w-0"
    : "opacity-100 max-w-full ml-3";

  return (
    <Layout.Sider
      collapsed={siderCollapsed}
      onCollapse={(collapsed) => setSiderCollapsed(collapsed)}
      collapsible
      width={175}
      theme="light"
    >
      <Flex
        justify="center"
        align="end"
        className="pt-3 pb-3 cursor-pointer"
        onClick={() =>
          window.open("https://github.com/AkiChase/scrcpy-mask", "_blank")
        }
      >
        <i
          className="w-8 h-8 bg-cover flex-shrink-0"
          style={{
            backgroundImage: `url(${logo})`,
          }}
        ></i>
        <div
          className={brandClass}
          style={{
            transition: "1s ease-in-out",
            whiteSpace: "nowrap",
            overflow: "hidden",
            textOverflow: "ellipsis",
          }}
        >
          <span className="color-text font-bold text-4">Scrcpy Mask</span>
        </div>
      </Flex>
      <Menu
        selectedKeys={[location.pathname]}
        onSelect={({ key }) => {
          navigate(key, { replace: true });
        }}
        items={[
          {
            key: "/devices",
            label: t("sider.devices"),
            icon: <IconFont type="icon-android" />,
          },
          {
            key: "/mappings",
            label: t("sider.mappings"),
            icon: <IconFont type="icon-keyboard" />,
          },
          {
            key: "/settings",
            label: t("sider.settings"),
            icon: <SettingFilled />,
          },
        ]}
      />
    </Layout.Sider>
  );
}
