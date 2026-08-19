# Provider Template

外部 API 适配器骨架。**默认用 Rust。** 本模板仅用于 ADR-0009 例外（该 provider 在 Rust 生态没有可用实现）。

## 结构

```
python/packages/provider/src/provider/{{NAME}}.py
```

## 骨架

```python
class {{CLASS}}Provider:
  def complete(self, prompt: str) -> str:
      raise NotImplementedError("skeleton")
```

## 禁止

- 密钥写入代码或契约
- Provider 内持久化
