---
name: opendesk-doc-comments
description: >-
  OpenDesk 强制文档注释与编码注释规范（作者 Xiaoman、创建时间、参数/返回值、Rust ///、TS JSDoc）。
  在编写、新增、修改、重构任何 OpenDesk 代码（Rust / TypeScript / React / subscription）时必须使用；
  用户提到注释、文档注释、JSDoc、///、作者、创建时间、编码规范时也必须使用。
  未写完公开 API 文档注释视为任务未完成。
---

# OpenDesk 文档注释（强制）

**写代码前先读本 Skill。公开 API 无完整文档注释 = 未完成，禁止交付。**

## 何时必须执行

- 新增 / 修改任何公开符号
- 新建文件、Class、Trait、导出函数
- 用户要求按编码规范实现

默认作者：`Xiaoman`  
创建时间：用对话里的 **Today's date**（`YYYY-MM-DD`），不要写死旧日期。

## 必须写文档注释的对象

- Class / Interface / Enum / Struct / Trait
- Public Function / Public Method
- Export 的变量、常量
- Rust：每个 `pub` 项（含 `pub` 字段若需说明）
- TS：所有 `export` 的类型、接口、类、函数

模块文件头也要有简短模块文档（`//!` 或文件级 JSDoc）。

## 文档注释必须包含

| 项 | 要求 |
|----|------|
| 功能说明 | 一句话 + 必要时用列表写「负责 / 功能」 |
| 作者 | Xiaoman |
| 创建时间 | YYYY-MM-DD |
| 参数说明 | 有参数则逐个写 |
| 返回值说明 | 有返回值则写；`Result` / `null` / `void` 说清语义 |
| 注意事项 | 有副作用、约束、失败模式时写 |
| 示例 | 复杂接口必须写 |

## TypeScript / JSDoc 模板

```ts
/**
 * 用户登录服务
 *
 * 负责：
 * - 登录
 * - Token 刷新
 * - Token 校验
 *
 * @author Xiaoman
 * @created 2026-07-16
 */
export class UserLoginService {}

/**
 * 根据用户 ID 查询用户信息
 *
 * @author Xiaoman
 * @created 2026-07-16
 *
 * @param userId - 用户 ID
 * @returns 用户对象；不存在返回 null
 */
export function findUserById(userId: string): User | null {}
```

字段用一行 `/** … */` 即可。

## Rust `///` 模板

```rust
//! 模块一句话说明。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-16

/// 用户仓库
///
/// 功能：
///
/// - 查询用户
/// - 创建用户
/// - 删除用户
///
/// 作者：Xiaoman
/// 创建时间：2026-07-16
pub struct UserRepository {}

/// 根据用户 ID 查询用户。
///
/// 作者：Xiaoman
/// 创建时间：2026-07-16
///
/// # 参数
///
/// * `user_id` - 用户 ID
///
/// # 返回值
///
/// 找到则 `Ok(Some)`，不存在 `Ok(None)`；存储失败返回错误。
pub fn find_user(user_id: &str) -> Result<Option<User>, StoreError> {}
```

## 行内注释（解释性）

复杂逻辑必须写「为什么」，禁止废话：

```ts
// 合并多页结果并按更新时间倒序；不能直接 concat，需要按 id 去重
```

禁止：`// 定义变量`、`// 调用函数`。

## 交付前自检（缺一不可）

- [ ] 本 diff 每个新增/改动的公开符号都有完整文档注释
- [ ] 含作者 Xiaoman + 当日创建时间
- [ ] 函数/方法含参数与返回值说明（若适用）
- [ ] 无空壳注释、无复制粘贴错误日期

**缺注释先补注释再结束回合，不要只实现功能。**

## 相关硬约束（写代码时一并遵守）

详见 [coding-standards.md](coding-standards.md)：OOP/Class、禁止业务 `unwrap`/`expect`/`any`、错误信息含何处/为何/如何解决、生产级质量。
