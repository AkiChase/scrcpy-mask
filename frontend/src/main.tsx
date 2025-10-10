import { createRoot } from "react-dom/client";
import "virtual:uno.css";
import "./index.css";
import router from "./router";
import "@ant-design/v5-patch-for-react-19";
import { Provider } from "react-redux";
import { store } from "./store/store.ts";
import { ConfigProvider, theme } from "antd";
import "./i18n";
import { RouterProvider } from "react-router-dom";

createRoot(document.getElementById("root")!).render(
  <Provider store={store}>
    <ConfigProvider
      theme={{
        algorithm: theme.darkAlgorithm,
        cssVar: true,
        hashed: false,
        token: {
          colorPrimary: "#b72a20",
          colorInfo: "#1677ff",
          colorLink: "#1677ff",
          colorSuccess: "#52c41a",
          colorError: "#de7c7d",
          colorWarning: "#c1840c",
          colorBgBase: "#060606",
        },
      }}
    >
      <RouterProvider router={router} />
    </ConfigProvider>
  </Provider>
);
