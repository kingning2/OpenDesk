# @desk/i18n

OpenDesk **通用** 多语言底座，基于 [i18next](https://www.i18next.com/) + [react-i18next](https://react.i18next.com/)。

## 边界

| 放在 `@desk/i18n` | 不放在 `@desk/i18n` |
|-------------------|---------------------|
| `createI18n` / Provider / hooks | 产品文案 JSON（放 app） |
| 插值 `{name}`；namespace 解析 | 后端用户提示（Rust 按 locale 自译） |

## App 文案目录约定

一路由一目录，目录内每种语言一个 JSON：

```
apps/desktop/src/i18n/locales/
  common/
    zh-cn.json
    en-us.json
  home/
    zh-cn.json
    en-us.json
  crawler/
    zh-cn.json
    en-us.json
```

- 目录名 = i18next **namespace**（通常对应路由 / Feature）
- 文件名 = 语言标签：`zh-cn.json` / `en-us.json`
- 调用：`t("crawler.status.idle")` → namespace=`crawler`，key=`status.idle`
- 未知首段回落到 `common`（如 `nav.home`、`shell.settings`）

## 使用

```ts
import { createI18n } from "@desk/i18n";
import { appLocaleResources } from "./load-resources";

export const appI18n = createI18n({
  defaultLocale: "zh-CN",
  defaultNS: "common",
  resources: appLocaleResources,
  localeLabels: { "zh-CN": "简体中文", "en-US": "English" },
  persistKey: "opendesk.locale",
});
```
