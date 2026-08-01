# Recipe: Add Contract

1. 在 `contracts/schema/v1/<feature>/<kind>/` 新建 Schema。
2. 定义稳定 `$id`、required、枚举和 `additionalProperties`。
3. 运行 `pnpm contracts:sync`。
4. 在 Rust 使用生成类型并实现边界。
5. 在 React 使用生成 TypeScript 类型。
6. 运行 `pnpm contracts:check`；兼容性变化追加 CHANGELOG/迁移说明。

禁止手改生成物。模板见 [`../templates/contract/`](../templates/contract/)。
