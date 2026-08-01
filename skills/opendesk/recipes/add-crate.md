# Recipe: Add Crate

只在现有 crate 无法清晰承载职责时新增：

1. 选择短名词并确认它是 Feature、Port 或 Infrastructure。
2. 从 [`../templates/crate/`](../templates/crate/) 复制最小 `Cargo.toml` 与 `lib.rs`。
3. 加入根 workspace；依赖优先复用 `[workspace.dependencies]`。
4. 不创建单实现 trait、空分层或“未来可能用”的模块。
5. 公开 API 写简洁中文 rustdoc。

验证：`cargo check -p <name> && pnpm lint:rust && pnpm check:architecture`。
