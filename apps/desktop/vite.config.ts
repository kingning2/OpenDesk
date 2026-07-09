import path from "node:path";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [react()],
  resolve: {
    alias: {
      "@desk/ui": path.resolve(__dirname, "../../packages/ui/src"),
      "@desk/platform": path.resolve(__dirname, "../../packages/platform/src"),
      "@desk/contracts": path.resolve(__dirname, "../../packages/contracts/src"),
      "@feature": path.resolve(__dirname, "./src/features"),
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
