import js from "@eslint/js";
import globals from "globals";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";
import tseslint from "typescript-eslint";

export default tseslint.config(
  {
    ignores: [
      "**/dist/**",
      "**/target/**",
      "**/node_modules/**",
      "**/.venv/**",
      "**/pnpm-lock.yaml",
      "tooling/strawberry-perl/**",
      "plugins/**", // 第三方参照项目（xianyu-auto-reply），非本仓库代码，不参与 lint
      "site/**", // 独立 Next.js 应用（自带 next build 类型检查与 lint），不参与根 lint
    ],
  },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ["tooling/scripts/**/*.mjs"],
    languageOptions: {
      ecmaVersion: 2020,
      globals: globals.node,
    },
  },
  {
    files: ["apps/**/*.{ts,tsx}", "packages/**/*.{ts,tsx}"],
    languageOptions: {
      ecmaVersion: 2020,
      globals: globals.browser,
    },
    plugins: {
      "react-hooks": reactHooks,
      "react-refresh": reactRefresh,
    },
    rules: {
      ...reactHooks.configs.recommended.rules,
      "react-refresh/only-export-components": [
        "warn",
        { allowConstantExport: true },
      ],
    },
  },
  {
    files: ["apps/desktop/src/features/**/*.{ts,tsx}"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          paths: [
            {
              name: "@tauri-apps/api",
              message:
                "Feature UI must call Rust via @desk/platform/ipc, not direct Tauri API.",
            },
            {
              name: "@tauri-apps/api/core",
              message:
                "Feature UI must call Rust via @desk/platform/ipc, not direct invoke().",
            },
          ],
        },
      ],
    },
  },
);
