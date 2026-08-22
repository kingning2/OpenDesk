//! 1688 IPC 链步骤 — 启用时追加命令并交给下一站。

#[cfg(platform_ali1688)]
#[macro_export]
macro_rules! platform_ipc_step_ali1688 {
    ($($prior:tt)*) => {
        $crate::platform_ipc_link_after_ali1688!(
            $($prior)*
            // 1688 专属 IPC 在此追加，例如：some_ali1688_command,
        )
    };
}
