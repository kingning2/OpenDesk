//! Application tracing subscriber initialization.

use tracing_subscriber::{fmt, EnvFilter};

pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("info,crawler=info,opendesk=debug,runtime=debug,app=debug")
    });

    let _ = fmt().with_env_filter(filter).with_target(true).try_init();
}
