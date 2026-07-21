import path from "node:path";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

/**
 * 拆分大体积依赖；其余交给 Rollup 默认策略，避免 vendor ↔ react 循环 chunk。
 *
 * @param id - Rollup 模块 id
 * @returns chunk 名；不强制拆分则返回 undefined
 */
function manualChunks(id: string): string | undefined {
  const normalized = id.replaceAll("\\", "/");

  if (!normalized.includes("/node_modules/")) {
    if (normalized.includes("/packages/ui/")) {
      return "desk-ui";
    }
    if (normalized.includes("/packages/platform/")) {
      return "desk-platform";
    }
    if (normalized.includes("/packages/i18n/")) {
      return "desk-i18n";
    }
    return undefined;
  }

  if (normalized.includes("/@xyflow/")) {
    return "vendor-xyflow";
  }
  if (normalized.includes("/lucide-react/") || normalized.includes("/lucide-react@")) {
    return "vendor-lucide";
  }
  if (normalized.includes("/@tauri-apps/")) {
    return "vendor-tauri";
  }
  if (normalized.includes("/react-router/")) {
    return "vendor-router";
  }
  if (
    normalized.includes("/react-dom/") ||
    normalized.includes("/scheduler/") ||
    /\/react\//.test(normalized)
  ) {
    return "vendor-react";
  }

  return undefined;
}

export default defineConfig(async () => ({
  plugins: [
    react({
      babel: {
        plugins: [["babel-plugin-react-compiler", {}]],
      },
    }),
    tailwindcss(),
  ],
  resolve: {
    alias: {
      "@desk/ui": path.resolve(__dirname, "../../packages/ui/src"),
      "@desk/platform": path.resolve(__dirname, "../../packages/platform/src"),
      "@desk/store": path.resolve(__dirname, "../../packages/store/src"),
      "@desk/i18n": path.resolve(__dirname, "../../packages/i18n/src"),
      "@desk/contracts": path.resolve(__dirname, "../../packages/contracts/src"),
      "@feature": path.resolve(__dirname, "./src/features"),
    },
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks,
      },
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
