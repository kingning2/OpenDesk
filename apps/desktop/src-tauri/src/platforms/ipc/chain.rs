//! IPC 注册链 — 平台顺序在此维护；新增平台只需追加一步 link。
//!
//! 顺序：`xianyu` → `ali1688` → `finish`
//!
//! 未启用的平台在此提供 **透传** 宏（平台模块可能整站 `cfg` 裁剪，链步骤不能放在平台 crate 内）。
//! 启用时的命令追加见各站 `platforms/<id>/ipc/chain.rs`。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-22

/// 共享 + core 命令入口 → 链首站。
#[macro_export]
macro_rules! platform_ipc_chain {
    ($($shared:tt)*) => {
        $crate::platform_ipc_step_xianyu!($($shared)*)
    };
}

/// 闲鱼之后 → 1688。
#[macro_export]
macro_rules! platform_ipc_link_after_xianyu {
    ($($cmds:tt)*) => {
        $crate::platform_ipc_step_ali1688!($($cmds)*)
    };
}

/// 1688 之后 → 收尾（新平台插在此 link 与 finish 之间）。
#[macro_export]
macro_rules! platform_ipc_link_after_ali1688 {
    ($($cmds:tt)*) => {
        $crate::platform_ipc_chain_finish!($($cmds)*)
    };
}

/// 链尾：`generate_handler!`。
#[macro_export]
macro_rules! platform_ipc_chain_finish {
    ($($cmds:tt)*) => {
        tauri::generate_handler![ $($cmds)* ]
    };
}

/// 闲鱼未启用：透传到下一站（`xianyu` 模块可能未链接）。
#[cfg(not(platform_xianyu))]
#[macro_export]
macro_rules! platform_ipc_step_xianyu {
    ($($prior:tt)*) => {
        $crate::platform_ipc_link_after_xianyu!($($prior)*)
    };
}

/// 1688 未启用：透传到下一站（`ali1688` 模块可能未链接）。
#[cfg(not(platform_ali1688))]
#[macro_export]
macro_rules! platform_ipc_step_ali1688 {
    ($($prior:tt)*) => {
        $crate::platform_ipc_link_after_ali1688!($($prior)*)
    };
}
