import { createBrowserRouter } from "react-router-dom";
import { lazy } from "react";
import LoadingWrapper from "./components/common/LoadingWrapper";
import App from "./App";
import NotFound from "./components/NotFound";

const Welcome = lazy(() => import("./components/Welcome"));
const Devices = lazy(() => import("./components/Devices"));
const Mappings = lazy(() => import("./components/mappings/Mappings"));
const Settings = lazy(() => import("./components/Settings"));

const router = createBrowserRouter([
  {
    path: "/",
    element: <App />,
    children: [
      {
        index: true,
        element: (
          <LoadingWrapper>
            <Welcome />
          </LoadingWrapper>
        ),
      },
      {
        path: "devices",
        element: (
          <LoadingWrapper>
            <Devices />
          </LoadingWrapper>
        ),
      },
      {
        path: "mappings",
        element: (
          <LoadingWrapper>
            <Mappings />
          </LoadingWrapper>
        ),
      },
      {
        path: "settings",
        element: (
          <LoadingWrapper>
            <Settings />
          </LoadingWrapper>
        ),
      },
    ],
  },
  {
    path: "*",
    element: <NotFound />,
  },
]);

export default router;
