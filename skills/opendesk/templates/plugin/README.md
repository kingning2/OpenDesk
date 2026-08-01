# Plugin Template

插件 manifest 必须声明版本与权限；Rust 适配层负责加载，业务层只依赖 Port。默认拒绝未声明的文件、网络和进程能力。

单一实现优先普通 crate，不为未来扩展预建插件框架。
