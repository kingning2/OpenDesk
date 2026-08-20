/// <reference types="vite/client" />

/** 编译期渠道平台 id — 由 Vite `define` 注入（见 `DINGDA_CHANNEL_PLATFORM`）。 */
declare const __DINGDA_CHANNEL_PLATFORM__:
  | "xianyu"
  | "xiaohongshu"
  | "douyin";

/** 编译期标题栏品牌中文名 — 由 Vite `define` 注入。 */
declare const __DINGDA_APP_BRAND_TITLE__: string;
