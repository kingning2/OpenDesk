mod channel_platform_cfg {
    include!("../../tooling/build/channel_platform_cfg.rs");
}

fn main() {
    channel_platform_cfg::emit_channel_platform_cfg("../../tooling/config/channel-platforms.json");
}
