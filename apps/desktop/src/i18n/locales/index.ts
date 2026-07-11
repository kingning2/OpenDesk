import en from "./en";
import zhCN from "./zh-CN";
import type { Locale, Messages } from "../types";

export const messagesByLocale: Record<Locale, Messages> = {
  "zh-CN": zhCN,
  en,
};

export const defaultLocale: Locale = "zh-CN";

export const localeLabels: Record<Locale, string> = {
  "zh-CN": "简体中文",
  en: "English",
};
