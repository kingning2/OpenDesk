//! OpenDesk 激活码签发 GUI（Slint）。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-16

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use slint::ComponentHandle;
use subscription_activation::issue::{ActivationIssuer, IssueRequest};
use subscription_activation::machine_code::compute_machine_code;

slint::include_modules!();

/// GUI 入口：打开签发窗口。
///
/// 作者：Xiaoman
/// 创建时间：2026-07-16
///
/// # 返回值
///
/// 窗口关闭后返回；平台错误向上传播。
fn main() -> Result<(), slint::PlatformError> {
    let window = ActivationGenWindow::new()?;
    let issuer = ActivationIssuer::new();

    {
        let ui = window.as_weak();
        window.on_fill_local_machine_code(move || {
            let Some(ui) = ui.upgrade() else {
                return;
            };
            match compute_machine_code() {
                Ok(code) => {
                    ui.set_machine_code(code.into());
                    ui.set_status_message("已填入本机设备码".into());
                }
                Err(error) => {
                    ui.set_status_message(format!("读取本机设备码失败: {error}").into());
                }
            }
        });
    }

    {
        let ui = window.as_weak();
        window.on_generate_clicked(move || {
            let Some(ui) = ui.upgrade() else {
                return;
            };
            if ui.get_busy() {
                return;
            }
            ui.set_busy(true);
            ui.set_status_message("正在生成…".into());

            let days_text = ui.get_days_text().to_string();
            let days = match days_text.trim().parse::<i64>() {
                Ok(value) if value > 0 => Some(value),
                _ => {
                    ui.set_busy(false);
                    ui.set_status_message("有效天数必须是正整数".into());
                    return;
                }
            };

            let request = IssueRequest {
                machine_code: ui.get_machine_code().to_string(),
                product: ui.get_product().to_string(),
                version: ui.get_version().to_string(),
                days,
                absolute_exp: None,
                private_key_path: {
                    let path = ui.get_private_key_path().to_string();
                    if path.trim().is_empty() {
                        None
                    } else {
                        Some(path)
                    }
                },
                output_path: ui.get_output_path().to_string(),
            };

            match issuer.issue(request) {
                Ok(result) => {
                    ui.set_token_preview(result.token.into());
                    ui.set_machine_code(result.machine_code.into());
                    ui.set_status_message(
                        format!("已生成: {}\n有效期从客户首次激活起算。", result.output_path)
                            .into(),
                    );
                }
                Err(error) => {
                    ui.set_status_message(format!("生成失败: {error}").into());
                }
            }
            ui.set_busy(false);
        });
    }

    window.run()
}
