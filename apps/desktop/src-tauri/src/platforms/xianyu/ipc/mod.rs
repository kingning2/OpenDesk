//! 闲鱼业务 IPC commands。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-18

pub mod account;
pub mod account_connection;
pub mod account_password;
pub mod account_qr;
pub mod address;
pub mod auto_reply_log;
pub mod blacklist;
pub mod card;
pub mod dashboard;
pub mod feedback;
pub mod filter;
pub mod item;
pub mod keyword;
pub mod notification;
pub mod order;
pub mod publish;
pub mod publish_batch;
pub mod publish_log;
pub mod publish_material;
pub mod rate;
pub mod risk;
pub mod setting;
pub mod site;

pub use account::{
    account_create, account_delete, account_list, account_set_status, account_update,
};
pub use account_connection::{account_connect, account_connection_state, account_disconnect};
pub use account_password::account_password_login;
pub use account_qr::{account_qr_cancel, account_qr_check, account_qr_start};
pub use address::{
    address_batch_delete, address_create, address_delete, address_list, address_update,
};
pub use auto_reply_log::auto_reply_log_list;
pub use blacklist::{
    blacklist_delete, blacklist_personal_create, blacklist_personal_list, blacklist_platform_list,
    blacklist_set_enabled,
};
pub use card::{card_create, card_delete, card_list, card_set_enabled, card_update};
pub use dashboard::dashboard_stats;
pub use feedback::{feedback_create, feedback_delete, feedback_list};
pub use filter::{filter_create, filter_delete, filter_list, filter_set_enabled, filter_update};
pub use item::{item_get, item_list, item_update};
pub use keyword::{keyword_add, keyword_delete, keyword_list, keyword_replace};
pub use notification::{
    notification_channel_create, notification_channel_delete, notification_channel_list,
    notification_channel_set_enabled, notification_channel_test, notification_channel_update,
    notification_delete, notification_list, notification_set,
};
pub use order::{
    order_create, order_delete, order_get, order_list, order_update_delivery, order_update_status,
};
pub use publish::{publish_capability, publish_single};
pub use publish_batch::{publish_batch_status, publish_batch_submit};
pub use publish_log::{publish_log_clear, publish_log_list};
pub use publish_material::{
    publish_material_batch_delete, publish_material_create, publish_material_delete,
    publish_material_list, publish_material_update,
};
pub use rate::{rate_buyer, rate_feedback_resolve};
pub use risk::{
    risk_config_get, risk_config_set, risk_log_clear, risk_log_clear_processing, risk_log_list,
    risk_log_today_rate,
};
pub use setting::{user_setting_get, user_setting_set, user_settings_get_all};
pub use site::{channel_close_site, channel_open_site};
