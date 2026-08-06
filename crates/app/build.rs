//! 注入编译期 target triple，供 `find_worker_binary` 拼 worker sidecar 文件名。

fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    println!("cargo:rustc-env=OPENDESK_WORKER_TARGET_TRIPLE={target}");
}
