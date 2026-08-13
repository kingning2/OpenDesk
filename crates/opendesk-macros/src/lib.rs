//! OpenDesk 过程宏。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-13

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, LitStr};

/// 为异步函数自动记录耗时日志（不限 IPC，任意 async fn 可用）。
///
/// 用法（若与其他属性叠用，写在更靠近函数的一侧）：
/// ```ignore
/// #[tauri::command]
/// #[timed]
/// pub async fn agent_ping(...) -> Result<..., String> {
///     // 原业务逻辑
/// }
/// ```
///
/// 可选自定义名称：`#[timed("custom_name")]`；默认使用函数名。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-13
#[proc_macro_attribute]
pub fn timed(attr: TokenStream, item: TokenStream) -> TokenStream {
    let name_override = if attr.is_empty() {
        None
    } else {
        Some(parse_macro_input!(attr as LitStr))
    };

    let func = parse_macro_input!(item as ItemFn);
    if func.sig.asyncness.is_none() {
        return syn::Error::new_spanned(func.sig.fn_token, "#[timed] 仅支持 async fn")
            .to_compile_error()
            .into();
    }

    let name = name_override
        .map(|lit| lit.value())
        .unwrap_or_else(|| func.sig.ident.to_string());

    let attrs = &func.attrs;
    let vis = &func.vis;
    let sig = &func.sig;
    let block = &func.block;

    quote! {
        #(#attrs)*
        #vis #sig {
            crate::timing::timed_run(#name, async #block).await
        }
    }
    .into()
}
