# Recipe: Add UseCase

1. 放入对应 Rust Feature 的 Application 层。
2. 输入/输出优先复用 Contract 或领域类型。
3. IO 通过 Port 注入；UseCase 不写 SQL、HTTP、文件或 Tauri API。
4. 返回明确 `Result`，为业务分支写最小测试。
5. 由 `crates/app`/Tauri command 适配 IPC。

模板见 [`../templates/usecase/`](../templates/usecase/)。
