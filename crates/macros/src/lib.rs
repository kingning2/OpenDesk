//! DingDa 过程宏。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-13

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, LitStr};

/// 为异步函数显式记录耗时日志（opt-in，按需标注，不默认用于全部 IPC）。
///
/// 用法（必须提供中文调用名）：
/// ```ignore
/// #[timed("插件安装")]
/// pub async fn plugin_install(...) -> Result<..., String> {
///     // 原业务逻辑
/// }
/// ```
///
/// 作者：Xiaoman
/// 创建时间：2026-08-13
#[proc_macro_attribute]
pub fn timed(attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);

    if attr.is_empty() {
        return syn::Error::new_spanned(
            &func.sig.ident,
            "#[timed] 必须提供中文调用名，例如 #[timed(\"插件安装\")]",
        )
        .to_compile_error()
        .into();
    }

    let name_lit = parse_macro_input!(attr as LitStr);
    if name_lit.value().trim().is_empty() {
        return syn::Error::new_spanned(&name_lit, "#[timed] 调用名不能为空")
            .to_compile_error()
            .into();
    }

    if func.sig.asyncness.is_none() {
        return syn::Error::new_spanned(func.sig.fn_token, "#[timed] 仅支持 async fn")
            .to_compile_error()
            .into();
    }

    let name = name_lit.value();

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
