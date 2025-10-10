import { defineConfig } from "vite";
import react from "@vitejs/plugin-react-swc";
import UnoCSS from "unocss/vite";

// https://vite.dev/config/
export default defineConfig({
  plugins: [UnoCSS(), react()],
  build: {
    outDir: "../assets/web",
  },
  server: {
    proxy: {
      // /api/* -> http://localhost:27799/api/*
      "/api": "http://localhost:27799",
    },
  },
});
