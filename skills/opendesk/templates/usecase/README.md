# UseCase Template

Application 层用例，无 IO。

## 生成

```bash
python skills/opendesk/scripts/create_usecase.py --crate chat --name list_threads
```

## 骨架

```rust
pub struct ListThreads;

impl ListThreads {
    pub fn execute(&self) -> Result<(), DomainError> {
        Ok(())
    }
}
```

## 禁止

SQL · HTTP · FS · Tauri · Python
