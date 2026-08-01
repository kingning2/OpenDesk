# UseCase Template

UseCase 放在 Rust Feature 的 Application 层，只依赖领域类型与 Port。禁止直接 SQL、HTTP、文件和 Tauri API。

公开 API 用简洁中文 rustdoc；相关步骤见 [`../../recipes/add-usecase.md`](../../recipes/add-usecase.md)。
