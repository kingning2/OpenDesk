# Architecture Decision Records

ADR 只记录长期有效、存在真实取舍的架构决定，不替代 Change Record。

## 存储方式

```text
decisions/
└── <domain>/
    └── adr-nnnn-short-name.md
```

## 使用条件

满足任一条件时创建 ADR：

- 改变跨层或跨端边界；
- 选择会长期绑定项目的框架或协议；
- 决策存在多个合理方案且未来可能被追问；
- 修改公共 API、兼容性或数据所有权。

ADR 被替代时不覆盖原文；新 ADR 使用 `supersedes` 指向旧记录。
