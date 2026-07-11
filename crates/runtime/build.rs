fn main() {
    let target = std::env::var("TARGET").expect("TARGET is set by Cargo");
    println!("cargo:rustc-env=OPENDESK_TARGET_TRIPLE={target}");
}
