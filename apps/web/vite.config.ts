import path from "node:path";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

/**
 * Web 前端 Vite 配置：复用 `@desk/*` 包与桌面 features 源码。
 *
 * dev server 端口 1422，`/api` 代理到 `opendesk-server`（默认 127.0.0.1:8899）。
 */
export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@desk/ui": path.resolve(__dirname, "../../packages/ui/src"),
      "@desk/platform": path.resolve(__dirname, "../../packages/platform/src"),
      "@desk/store": path.resolve(__dirname, "../../packages/store/src"),
      "@desk/i18n": path.resolve(__dirname, "../../packages/i18n/src"),
      "@desk/contracts": path.resolve(__dirname, "../../packages/contracts/src"),
      "@desk/utils": path.resolve(__dirname, "../../packages/utils/src"),
      "@feature": path.resolve(__dirname, "../desktop/src/features"),
    },
  },
  server: {
    port: 1422,
    strictPort: true,
    proxy: {
      "/api": {
        target: process.env.OPENDESK_SERVER_URL ?? "http://127.0.0.1:8899",
        changeOrigin: true,
      },
    },
  },
  clearScreen: false,
});
