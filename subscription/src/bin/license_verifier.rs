//! 供主程序调用的校验 exe：OpenSSL 验签 + 机器码 + 过期 + 挑战应答。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-16

use clap::{Parser, Subcommand};
use subscription_activation::{machine_code::compute_machine_code, verify::verify_local};

#[derive(Parser, Debug)]
#[command(name = "license-verifier")]
#[command(about = "License verifier for OpenDesk (vendored OpenSSL)")]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Compute local machineCode from Windows hardware IDs
    GenMachineCode,
    /// Full verify: RSA signature + machineCode + expiration (+ optional nonce)
    Verify {
        #[arg(long)]
        token: Option<String>,
        #[arg(long)]
        key_file: Option<String>,
        #[arg(long)]
        now: Option<i64>,
        /// Challenge nonce from host; when set, response includes attestation HMAC
        #[arg(long)]
        nonce: Option<String>,
        /// Directory for license.activated_at.json (required for --days licenses)
        #[arg(long)]
        state_dir: Option<String>,
    },
}

fn run(cli: Cli) -> Result<i32, String> {
    match cli.cmd {
        Commands::GenMachineCode => {
            let code = compute_machine_code()?;
            println!("{code}");
            Ok(0)
        }
        Commands::Verify {
            token,
            key_file,
            now,
            nonce,
            state_dir,
        } => {
            let (output, exit) = verify_local(token, key_file, now, nonce, state_dir)?;
            println!(
                "{}",
                serde_json::to_string(&output)
                    .map_err(|e| format!("json serialize failed: {e}"))?
            );
            Ok(exit)
        }
    }
}

fn main() {
    let cli = Cli::parse();
    match run(cli) {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("ERROR: {e}");
            std::process::exit(2);
        }
    }
}
