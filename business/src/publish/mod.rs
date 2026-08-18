//! 商品发布 — 单品发布执行编排。
//!
//! 对齐 Python 版 `publish_execution_service.py` 的核心流程：
//! 1. 校验账号存在 + Cookie；
//! 2. 解析发布地址（收货/发货地址解析）；
//! 3. 创建发布日志（publishing）；
//! 4. 账号能力检测（鱼小铺 vs 普通卖家 → 不同发布入口）；
//! 5. 执行发布，更新日志（success/failed + item_url/item_id）；
//! 6. 发布成功后同步账号商品。
//!
//! 平台发布/地址/同步全部经 [`gateway::PublishGateway`] Port 注入，编排纯逻辑可单测。

pub mod address_store;
pub mod batch;
pub mod gateway;
pub mod log_store;
pub mod material_store;
pub mod service;

pub use address_store::{AddressQuery, AddressService, AddressStore, AddressType, PublishAddress};
pub use batch::{BatchAccountStatus, BatchService, BatchStore, BatchTask};
pub use gateway::{AccountCapability, PublishGateway, PublishResult, SyncInfo};
pub use log_store::{
    PublishLog, PublishLogQuery, PublishLogService, PublishLogStatus, PublishLogStore,
};
pub use material_store::{
    PublishMaterial, PublishMaterialQuery, PublishMaterialService, PublishMaterialStore,
};
pub use service::{PublishRequest, PublishService, PublishServiceResult};
