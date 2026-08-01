# Recipe: Add Plugin

先确认动态插件确有需求；单一实现优先普通 Rust crate。

需要插件时：

1. 定义最小 manifest Contract 与权限。
2. 在 Rust 适配层加载，业务只依赖 Port。
3. 默认拒绝未声明的文件、网络和进程权限。
4. 记录加载失败但不泄露路径或凭据。
5. 为版本不兼容和权限拒绝写测试。

模板见 [`../templates/plugin/`](../templates/plugin/)。
