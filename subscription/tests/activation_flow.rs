use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn exe_path() -> &'static OsStr {
    // Provided by Cargo for integration tests of a binary crate.
    env!("CARGO_BIN_EXE_subscription-activation-generator").as_ref()
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn private_key_path() -> PathBuf {
    manifest_dir().join("keys").join("private_key.pem")
}

fn run_with_cwd(
    exe: &OsStr,
    args: &[&str],
    env_kv: &[(&str, String)],
    cwd: &PathBuf,
) -> (i32, String, String) {
    let mut cmd = Command::new(exe);
    cmd.args(args);
    cmd.current_dir(cwd);
    for (k, v) in env_kv {
        cmd.env(k, v);
    }

    let out = cmd.output().expect("failed to spawn activation generator");
    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    (code, stdout, stderr)
}

fn assert_hex_sha256(s: &str) {
    assert!(
        s.len() == 64
            && s.chars()
                .all(|c| c.is_ascii_hexdigit() && c.to_ascii_lowercase() == c),
        "expected 64-lowercase-hex sha256, got: {s}"
    );
}

#[test]
fn activation_flow_works_end_to_end() {
    let exe = exe_path();
    let key_path = private_key_path();
    assert!(
        key_path.is_file(),
        "missing private key file: {}",
        key_path.display()
    );

    // Use crate directory as CWD so the generator can find `./keys/private_key.pem`.
    let crate_dir = manifest_dir();

    // 1) gen-machine-code
    let (code1, machine_code, stderr1) = run_with_cwd(exe, &["gen-machine-code"], &[], &crate_dir);
    assert_eq!(code1, 0, "gen-machine-code failed: {stderr1}");
    assert_hex_sha256(&machine_code);

    // 2) generate-activation-code -> binary key file
    let out_key = crate_dir.join("target").join(format!(
        "test-license-{}.key",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    let out_key_str = out_key.to_string_lossy().to_string();

    let (code2, out_path, stderr2) = run_with_cwd(
        exe,
        &[
            "generate-activation-code",
            "--machine-code",
            machine_code.as_str(),
            "--days",
            "3",
            "--product",
            "supportflow",
            "--version",
            "1",
            "--out",
            out_key_str.as_str(),
        ],
        &[],
        &crate_dir,
    );
    assert_eq!(code2, 0, "generate-activation-code failed: {stderr2}");
    assert!(!out_path.is_empty(), "expected output key path, got empty");
    assert!(
        out_key.is_file(),
        "expected key file exists: {}",
        out_key.display()
    );
    let key_bytes = std::fs::read(&out_key).expect("read generated key file");
    assert!(key_bytes.len() > 16, "key file too small");
    assert_eq!(&key_bytes[0..4], b"SFLK", "unexpected key file magic");
    assert!(
        std::str::from_utf8(&key_bytes).is_err(),
        "key file should not be plain UTF-8 token text"
    );

    // Negative 1: missing private key -> should fail with exit code != 0
    let missing_key_cwd = manifest_dir().join("target");
    let (code5, _, stderr5) = run_with_cwd(
        exe,
        &[
            "generate-activation-code",
            "--machine-code",
            machine_code.as_str(),
            "--days",
            "1",
            "--product",
            "supportflow",
            "--version",
            "1",
        ],
        &[],
        &missing_key_cwd,
    );
    assert_ne!(code5, 0, "expected failure when private key is missing");
    assert!(
        stderr5.contains("private key file not found"),
        "expected missing private key error, got: {stderr5}"
    );

    // Negative 2: --days + --exp are mutually exclusive
    let (code6, _, stderr6) = run_with_cwd(
        exe,
        &[
            "generate-activation-code",
            "--machine-code",
            machine_code.as_str(),
            "--days",
            "1",
            "--exp",
            "2026-12-31T23:59:59Z",
            "--product",
            "supportflow",
            "--version",
            "1",
        ],
        &[],
        &crate_dir,
    );
    assert_ne!(code6, 0, "expected failure for --days + --exp");
    assert!(
        stderr6.contains("use either --exp or --days"),
        "unexpected error for mutually exclusive args: {stderr6}"
    );

    // 3) parse-license via key file
    let (code3, pretty, stderr3) = run_with_cwd(
        exe,
        &["parse-license", "--key-file", out_key_str.as_str()],
        &[],
        &crate_dir,
    );
    assert_eq!(code3, 0, "parse-license --key-file failed: {stderr3}");
    assert!(
        pretty.contains("machineCode"),
        "expected parsed claims to contain machineCode, got: {pretty}"
    );

    // 4) check-local via key file
    let (code4, _, stderr4) = run_with_cwd(
        exe,
        &["check-local", "--key-file", out_key_str.as_str()],
        &[],
        &crate_dir,
    );
    assert_eq!(code4, 0, "check-local --key-file failed: {stderr4}");

    // Negative 3: invalid token parse command should still fail
    let (code7, _, stderr7) = run_with_cwd(
        exe,
        &["parse-license", "--token", "not-a-token"],
        &[],
        &crate_dir,
    );
    assert_ne!(code7, 0, "expected parse-license failure for invalid token");
    assert!(
        stderr7.contains("token base64url decode failed")
            || stderr7.contains("token json decode failed"),
        "unexpected parse error: {stderr7}"
    );
}
