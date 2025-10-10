import { defineConfig, presetWind3 } from "unocss";

export default defineConfig({
  presets: [presetWind3()],
  theme: {
    colors: {
      text: {
        DEFAULT: "var(--ant-color-text)",
        secondary: "var(--ant-color-text-secondary)",
        tertiary: "var(--ant-color-text-tertiary)",
        quaternary: "var(--ant-color-text-quaternary)",
      },
      primary: {
        DEFAULT: "var(--ant-color-primary)",
        hover: "var(--ant-color-primary-hover)",
        active: "var(--ant-color-primary-active)",
        bg: {
          DEFAULT: "var(--ant-color-primary-bg)",
          hover: "var(--ant-color-primary-bg-hover)",
        },
        border: {
          DEFAULT: "var(--ant-color-primary-border)",
          hover: "var(--ant-color-primary-border-hover)",
        },
        text: {
          DEFAULT: "var(--ant-color-primary-text)",
          hover: "var(--ant-color-primary-text-hover)",
          active: "var(--ant-color-primary-text-active)",
        },
      },
      success: {
        DEFAULT: "var(--ant-color-success)",
        hover: "var(--ant-color-success-hover)",
        active: "var(--ant-color-success-active)",
        bg: {
          DEFAULT: "var(--ant-color-success-bg)",
          hover: "var(--ant-color-success-bg-hover)",
        },
        border: {
          DEFAULT: "var(--ant-color-success-border)",
          hover: "var(--ant-color-success-border-hover)",
        },
        text: {
          DEFAULT: "var(--ant-color-success-text)",
          hover: "var(--ant-color-success-text-hover)",
          active: "var(--ant-color-success-text-active)",
        },
      },
      error: {
        DEFAULT: "var(--ant-color-error)",
        hover: "var(--ant-color-error-hover)",
        active: "var(--ant-color-error-active)",

        bg: {
          DEFAULT: "var(--ant-color-error-bg)",
          hover: "var(--ant-color-error-bg-hover)",
        },
        text: {
          DEFAULT: "var(--ant-color-error-text)",
          hover: "var(--ant-color-error-text-hover)",
          active: "var(--ant-color-error-text-active)",
        },
      },
      warning: {
        DEFAULT: "var(--ant-color-warning)",
        hover: "var(--ant-color-warning-hover)",
        active: "var(--ant-color-warning-active)",
        bg: {
          DEFAULT: "var(--ant-color-warning-bg)",
          hover: "var(--ant-color-warning-bg-hover)",
        },
        text: {
          DEFAULT: "var(--ant-color-warning-text)",
          hover: "var(--ant-color-warning-text-hover)",
          active: "var(--ant-color-warning-text-active)",
        },
      },
      info: {
        DEFAULT: "var(--ant-color-info)",
        hover: "var(--ant-color-info-hover)",
        active: "var(--ant-color-info-active)",
        bg: {
          DEFAULT: "var(--ant-color-info-bg)",
          hover: "var(--ant-color-info-bg-hover)",
        },
        text: {
          DEFAULT: "var(--ant-color-info-text)",
          hover: "var(--ant-color-info-text-hover)",
          active: "var(--ant-color-info-text-active)",
        },
      },
    },
  },
  safelist: [
    "color-info-text",
    "hover:color-info-text-hover",
    "active:color-info-text-active",
    "color-error-text",
    "hover:color-error-text-hover",
    "active:color-error-text-active",
    "color-primary-text",
    "hover:color-primary-text-hover",
    "active:color-primary-text-active",
    "color-success-text",
    "hover:color-success-text-hover",
    "active:color-success-text-active",
    "color-warning-text",
    "hover:color-warning-text-hover",
    "active:color-warning-text-active",
  ],
});
