fn main() {
    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".into());
    println!("cargo:rustc-env=OPENDESK_TARGET_TRIPLE={target}");
}
