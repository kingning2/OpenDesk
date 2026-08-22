//! 闲鱼在售商品拉取 — 卖家主页商品列表同步。
//!
//! 接口：`mtop.idle.web.xyh.item.list` v1.0（发现过程见
//! [`skills/dingda/guides/xianyu-mtop-discovery.md`](../../../../skills/dingda/guides/xianyu-mtop-discovery.md)）。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-20

use common::DingDaResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::xianyu::core::cookie::{my_id, parse_cookies};
use crate::xianyu::core::cookies::credential_to_cookie_header;
use crate::xianyu::core::mtop::{MtopClient, MtopRequest};

/// 平台侧商品摘要（同步入库前）。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-20
#[derive(Debug, Clone, PartialEq)]
pub struct PlatformItem {
    /// 闲鱼商品 ID。
    pub item_id: String,
    /// 商品标题。
    pub title: String,
    /// 售价（元）。
    pub price: f64,
    /// 商品描述（列表接口通常为空）。
    pub desc: String,
}

/// 拉取指定卖家在售商品（自动翻页，最多 `max_pages` 页）。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-20
///
/// # 参数
///
/// * `cookie_str` — 账号 Cookie 原文
/// * `user_id` — 卖家 userId（通常取 `unb`）
/// * `max_pages` — 最大翻页数（0 表示默认 50 页）
///
/// # 返回值
///
/// 成功返回 `(商品列表, 最新 Cookie)`；mtop 可能通过 set-cookie 刷新签名 token。
pub async fn fetch_seller_items(
    cookie_str: &str,
    user_id: &str,
    max_pages: u32,
) -> DingDaResult<(Vec<PlatformItem>, String)> {
    let user_id = user_id.trim();
    if user_id.is_empty() {
        return Err("userId 不能为空".into());
    }

    let cookies = parse_cookies(cookie_str);
    if my_id(&cookies).is_none() {
        return Err("cookie 缺少 unb，无法拉取商品".into());
    }

    let client = MtopClient::new(&credential_to_cookie_header(cookie_str))?;
    let page_limit = if max_pages == 0 { 50 } else { max_pages };
    let mut all_items = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    for page_number in 1..=page_limit {
        let request = MtopRequest::new(
            "mtop.idle.web.xyh.item.list",
            "1.0",
            serde_json::json!({
                "userId": user_id,
                "pageNumber": page_number,
                "scene": "seller_home",
                "pageSize": 10,
            }),
        );
        let response = client.call(&request).await?;
        if !response.success() {
            return Err(format!("商品列表接口未成功: {}", response.ret).into());
        }

        let page_items = parse_list_page(response.data().unwrap_or(&Value::Null));
        if page_items.is_empty() {
            break;
        }

        let mut added = 0u32;
        for item in page_items {
            if seen_ids.insert(item.item_id.clone()) {
                all_items.push(item);
                added += 1;
            }
        }
        if added == 0 {
            break;
        }
    }

    Ok((all_items, client.cookie().await))
}

/// 闲鱼商品详情（`mtop.taobao.idle.pc.detail` 解析结果）。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-20
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlatformItemDetail {
    /// 商品 ID。
    pub item_id: String,
    /// 标题。
    pub title: String,
    /// 详情描述（优先 shareInfoJsonString 内文案）。
    pub desc: String,
    /// 售价（元）。
    pub price: f64,
    /// 原价（元）。
    pub original_price: Option<f64>,
    /// 图片 URL 列表。
    pub images: Vec<String>,
    /// 想要人数。
    pub want_count: Option<u32>,
    /// 浏览次数。
    pub browse_count: Option<u32>,
    /// 闲鱼详情页链接。
    pub item_url: String,
}

/// 拉取单个商品详情。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-20
///
/// # 参数
///
/// * `cookie_str` — 账号 Cookie 原文
/// * `item_id` — 闲鱼商品 ID
///
/// # 返回值
///
/// 成功返回 `(PlatformItemDetail, 最新 Cookie)`。
pub async fn fetch_item_detail(
    cookie_str: &str,
    item_id: &str,
) -> DingDaResult<(PlatformItemDetail, String)> {
    let item_id = item_id.trim();
    if item_id.is_empty() {
        return Err("itemId 不能为空".into());
    }

    let cookies = parse_cookies(cookie_str);
    if my_id(&cookies).is_none() {
        return Err("cookie 缺少 unb，无法拉取商品详情".into());
    }

    let client = MtopClient::new(&credential_to_cookie_header(cookie_str))?;
    let request = MtopRequest::new(
        "mtop.taobao.idle.pc.detail",
        "1.0",
        serde_json::json!({ "itemId": item_id }),
    );
    let response = client.call(&request).await?;
    if !response.success() {
        return Err(format!("商品详情接口未成功: {}", response.ret).into());
    }

    let detail = parse_item_detail(response.data().unwrap_or(&Value::Null), item_id)?;
    Ok((detail, client.cookie().await))
}

/// 解析 `mtop.taobao.idle.pc.detail` 的 `data` 节点。
fn parse_item_detail(data: &Value, item_id: &str) -> DingDaResult<PlatformItemDetail> {
    let item_do = data
        .get("itemDO")
        .ok_or_else(|| "商品详情响应缺少 itemDO".to_string())?;

    let mut title = text_field(item_do, &["title"]).unwrap_or_default();
    let mut desc = text_field(item_do, &["desc"]).unwrap_or_default();
    let mut images = extract_image_urls(item_do);

    if let Some(share_json) = item_do
        .pointer("/shareData/shareInfoJsonString")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
    {
        if let Ok(inner) = serde_json::from_str::<Value>(share_json) {
            let main_params = inner.pointer("/contentParams/mainParams").unwrap_or(&inner);
            if let Some(content) = text_field(main_params, &["content"]) {
                if !content.is_empty() {
                    desc = content;
                }
            }
            let share_images = main_params
                .get("images")
                .and_then(Value::as_array)
                .map(|list| {
                    list.iter()
                        .filter_map(|entry| {
                            entry
                                .get("image")
                                .or_else(|| entry.get("url"))
                                .and_then(Value::as_str)
                                .map(str::to_string)
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if !share_images.is_empty() {
                images = share_images;
            }
        }
    }

    if title.is_empty() {
        title = desc.lines().next().unwrap_or("").trim().to_string();
    }

    let price = parse_price_value(item_do.get("soldPrice"))
        .or_else(|| parse_price_value(item_do.pointer("/priceInfo/price")))
        .unwrap_or(0.0);
    let original_price = parse_price_value(item_do.get("originalPrice"));

    Ok(PlatformItemDetail {
        item_id: item_id.to_string(),
        title,
        desc,
        price,
        original_price,
        images,
        want_count: u32_field(item_do, &["wantCnt", "wantCount"]),
        browse_count: u32_field(item_do, &["browseCnt", "browseCount"]),
        item_url: format!("{}{item_id}", common::constants::xianyu::ITEM_URL_PREFIX),
    })
}

fn text_field(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| value.get(*key))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

fn u32_field(value: &Value, keys: &[&str]) -> Option<u32> {
    keys.iter().find_map(|key| {
        value.get(*key).and_then(|entry| {
            entry
                .as_u64()
                .map(|number| number as u32)
                .or_else(|| entry.as_str()?.parse().ok())
        })
    })
}

fn parse_price_value(value: Option<&Value>) -> Option<f64> {
    let value = value?;
    if let Some(number) = value.as_f64() {
        return Some(number);
    }
    value.as_str()?.parse().ok()
}

fn extract_image_urls(item_do: &Value) -> Vec<String> {
    let mut urls = Vec::new();
    if let Some(list) = item_do.get("imageInfos").and_then(Value::as_array) {
        for entry in list {
            if let Some(url) = entry
                .get("url")
                .or_else(|| entry.get("image"))
                .and_then(Value::as_str)
            {
                urls.push(url.to_string());
            }
        }
    }
    urls
}

/// 解析单页 `data` 节点中的商品列表。
fn parse_list_page(data: &Value) -> Vec<PlatformItem> {
    let entries = data
        .get("cardList")
        .or_else(|| data.get("items"))
        .and_then(Value::as_array);

    let Some(entries) = entries else {
        return Vec::new();
    };

    entries.iter().filter_map(parse_list_item).collect()
}

/// 解析列表项中的商品 ID / 标题 / 价格。
fn parse_list_item(entry: &Value) -> Option<PlatformItem> {
    let item_id = extract_item_id(entry)?;
    Some(PlatformItem {
        item_id,
        title: extract_title(entry),
        price: extract_price(entry),
        desc: String::new(),
    })
}

fn extract_item_id(entry: &Value) -> Option<String> {
    entry
        .pointer("/cardData/detailParams/itemId")
        .or_else(|| entry.pointer("/data/itemId"))
        .or_else(|| entry.get("itemId"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn extract_title(entry: &Value) -> String {
    entry
        .pointer("/cardData/main/title")
        .or_else(|| entry.pointer("/cardData/detailParams/title"))
        .or_else(|| entry.pointer("/data/title"))
        .or_else(|| entry.get("title"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn extract_price(entry: &Value) -> f64 {
    const PATHS: [&str; 6] = [
        "/cardData/main/soldPrice",
        "/cardData/detailParams/soldPrice",
        "/data/soldPrice",
        "/soldPrice",
        "/cardData/main/price",
        "/price",
    ];
    for path in PATHS {
        if let Some(value) = entry.pointer(path) {
            if let Some(number) = value.as_f64() {
                return number;
            }
            if let Some(text) = value.as_str() {
                if let Ok(number) = text.parse::<f64>() {
                    return number;
                }
            }
        }
    }
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_card_list_entry() {
        let data = json!({
            "cardList": [{
                "cardData": {
                    "main": { "title": "二手手机", "soldPrice": "128.5" },
                    "detailParams": { "itemId": "1234567890" }
                }
            }]
        });
        let items = parse_list_page(&data);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].item_id, "1234567890");
        assert_eq!(items[0].title, "二手手机");
        assert!((items[0].price - 128.5).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_items_array_entry() {
        let data = json!({
            "items": [{ "itemId": "99", "title": "耳机", "soldPrice": 50 }]
        });
        let items = parse_list_page(&data);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].item_id, "99");
    }

    #[test]
    fn skip_entry_without_item_id() {
        let data = json!({ "cardList": [{ "cardData": { "main": { "title": "无 ID" } } }] });
        assert!(parse_list_page(&data).is_empty());
    }

    #[test]
    fn parse_item_detail_from_item_do() {
        let data = json!({
            "itemDO": {
                "title": "二手手机",
                "soldPrice": "199.5",
                "originalPrice": "299",
                "desc": "备用描述",
                "wantCnt": 8,
                "browseCnt": 120,
                "imageInfos": [{ "url": "https://img.example/1.jpg" }]
            }
        });
        let detail = parse_item_detail(&data, "123").expect("parse");
        assert_eq!(detail.item_id, "123");
        assert_eq!(detail.title, "二手手机");
        assert_eq!(detail.desc, "备用描述");
        assert!((detail.price - 199.5).abs() < f64::EPSILON);
        assert_eq!(detail.images.len(), 1);
        assert_eq!(detail.want_count, Some(8));
    }

    #[test]
    fn parse_item_detail_from_share_json_string() {
        let share = json!({
            "contentParams": {
                "mainParams": {
                    "content": "真正文案\n第二行",
                    "images": [{ "image": "https://img.example/hd.jpg" }]
                }
            }
        });
        let data = json!({
            "itemDO": {
                "title": "",
                "soldPrice": 50,
                "shareData": {
                    "shareInfoJsonString": share.to_string()
                }
            }
        });
        let detail = parse_item_detail(&data, "999").expect("parse");
        assert_eq!(detail.desc, "真正文案\n第二行");
        assert_eq!(
            detail.images,
            vec!["https://img.example/hd.jpg".to_string()]
        );
    }
}
