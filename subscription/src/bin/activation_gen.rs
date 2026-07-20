use clap::{Parser, Subcommand};
use subscription_activation::{
    claims::{now_ts, parse_activation_code},
    issue::{ActivationIssuer, IssueRequest},
    key_file::resolve_token_input,
    machine_code::compute_machine_code,
};

#[derive(Parser, Debug)]
#[command(name = "activation-gen")]
#[command(
    about = "Offline activation key generator (Rust). Prefer activation-gen-gui for a window."
)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Compute local machineCode from Windows hardware IDs
    GenMachineCode,
    /// Generate binary activation key file using RSA-PSS(SHA256)
    ///
    /// `--days`: validity starts at first successful activation on the device.
    /// `--exp`: absolute calendar expiry (unchanged by activation time).
    GenerateActivationCode {
        #[arg(long)]
        machine_code: Option<String>,
        /// Absolute expiry (unix / RFC3339 / YYYY-MM-DD). Mutually exclusive with --days.
        #[arg(long)]
        exp: Option<String>,
        /// Valid for N days starting at first activation (not at issuance). Mutually exclusive with --exp.
        #[arg(long)]
        days: Option<i64>,
        #[arg(long, default_value = "supportflow")]
        product: String,
        #[arg(long = "version", default_value = "1")]
        version: String,
        #[arg(long)]
        private_key: Option<String>,
        #[arg(long, default_value = "license.key")]
        out: String,
    },
    /// Parse activation claims from token or key file
    ParseLicense {
        #[arg(long)]
        token: Option<String>,
        #[arg(long)]
        key_file: Option<String>,
    },
    /// Compare token/key exp + machineCode with local machine (no signature check)
    CheckLocal {
        #[arg(long)]
        token: Option<String>,
        #[arg(long)]
        key_file: Option<String>,
    },
}

async fn cmd_generate_activation_code(
    machine_code_arg: Option<String>,
    exp_arg: Option<String>,
    days_arg: Option<i64>,
    product: String,
    version: String,
    private_key: Option<String>,
    out: String,
) -> Result<(), String> {
    let issuer = ActivationIssuer::new();
    let request = IssueRequest {
        machine_code: machine_code_arg.unwrap_or_default(),
        product,
        version,
        days: days_arg,
        absolute_exp: exp_arg,
        private_key_path: private_key,
        output_path: out,
    };
    let result = tokio::task::spawn_blocking(move || issuer.issue(request))
        .await
        .map_err(|e| format!("activation task failed: {e}"))??;
    println!("{}", result.output_path);
    Ok(())
}

fn cmd_parse_license(token: Option<String>, key_file: Option<String>) -> Result<(), String> {
    let token = resolve_token_input(token, key_file)?;
    let claims = parse_activation_code(&token)?;
    let pretty =
        serde_json::to_string_pretty(&claims).map_err(|e| format!("json pretty failed: {e}"))?;
    println!("{pretty}");
    Ok(())
}

fn cmd_check_local(token: Option<String>, key_file: Option<String>) -> Result<i32, String> {
    let token = resolve_token_input(token, key_file)?;
    let claims = parse_activation_code(&token)?;
    let local_machine_code = compute_machine_code()?;
    let now = now_ts();
    let machine_match = claims.machine_code == local_machine_code;

    // check-local 不做签名、不写激活状态；时长授权仅提示「尚未激活则未开始计时」。
    let (token_valid_time, token_expired_at, note) = if let Some(duration) = claims.duration_secs {
        (
            true,
            now.saturating_add(duration),
            Some(
                "durationSecs present: clock starts at first successful activation, not at issuance"
                    .to_string(),
            ),
        )
    } else {
        (now <= claims.exp, claims.exp, None)
    };

    let output = serde_json::json!({
        "product": claims.product,
        "tokenValidTime": token_valid_time,
        "tokenExpiredAt": token_expired_at,
        "durationSecs": claims.duration_secs,
        "now": now,
        "machineMatch": machine_match,
        "tokenMachineCode": claims.machine_code,
        "localMachineCode": local_machine_code,
        "signatureVerified": false,
        "note": note
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&output).map_err(|e| format!("json pretty failed: {e}"))?
    );
    Ok(if machine_match && token_valid_time {
        0
    } else {
        1
    })
}

async fn run(cli: Cli) -> Result<i32, String> {
    match cli.cmd {
        Commands::GenMachineCode => {
            let code = tokio::task::spawn_blocking(compute_machine_code)
                .await
                .map_err(|e| format!("machine code task failed: {e}"))??;
            println!("{code}");
            Ok(0)
        }
        Commands::GenerateActivationCode {
            machine_code,
            exp,
            days,
            product,
            version,
            private_key,
            out,
        } => {
            cmd_generate_activation_code(
                machine_code,
                exp,
                days,
                product,
                version,
                private_key,
                out,
            )
            .await?;
            Ok(0)
        }
        Commands::ParseLicense { token, key_file } => {
            cmd_parse_license(token, key_file)?;
            Ok(0)
        }
        Commands::CheckLocal { token, key_file } => {
            let exit = tokio::task::spawn_blocking(move || cmd_check_local(token, key_file))
                .await
                .map_err(|e| format!("check-local task failed: {e}"))??;
            Ok(exit)
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match run(cli).await {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("ERROR: {e}");
            std::process::exit(2);
        }
    }
}
