# OpenDesk 编码规范（配套）

与 [SKILL.md](SKILL.md) 配套；实现代码时与文档注释一起遵守。

## 结构

1. 优先 OOP；有状态 / 职责 / 生命周期 → Class，禁止一堆散函数堆业务。
2. 单一职责；禁止 God Object；复杂逻辑拆 private。
3. 禁止复制粘贴；重复逻辑抽取。

## 命名

见名知意。禁止 `a` / `tmp` / `aaa`。布尔用 `is` / `has` / `can` / `should` 前缀。

## 函数

一件事；>80 行考虑拆；参数 >5 封装对象；优先提前 return。

## 类

文档注释 + 成员 + 构造 + 公共方法（前）+ 私有方法（后）。

## 错误

禁止空 catch、业务路径 `unwrap`/`expect`、吞错。错误须说明：哪里、为什么、如何解决。Rust 业务用 `Result<T, E>`。

## 日志

带上下文（id、路径、耗时）。禁止无上下文的 `console.log("error")`。

## 语言

### Rust

- 业务路径禁用 `unwrap`/`expect`
- 统一 `Result`
- 每个 `pub struct` / `enum` / `trait` / `fn` 必须有 `///`

### TypeScript

- `strict`；禁止 `any`（用 `unknown` / 泛型）
- 所有 export 必须有 JSDoc
- 有状态业务优先 class；一文件一核心职责

## 质量门

未写完公开 API 文档注释 = 任务未完成。优先可维护、可测试、低耦合的生产级代码，而非最短 Demo。
