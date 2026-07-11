# Change Records

## 存储方式

```text
changes/
└── YYYY/
    └── MM/
        └── chg-yyyymmdd-nnn-short-name.md
```

每次代码、契约、配置或依赖修改使用一个独立文件。月份目录按需创建，不预生成空目录。

## 查找方式

- 未完成记录从 `registry/ACTIVE.md` 进入；
- 已知 ID 可按 ID 中的年月直接定位；
- 历史调查使用文件名搜索，不维护无限增长的全局历史索引。

## 完成后的规则

完成记录保留在原年月目录，不移动、不汇总正文。只允许修正链接或明显笔误；若结论改变，应新建 Change 或 ADR。
