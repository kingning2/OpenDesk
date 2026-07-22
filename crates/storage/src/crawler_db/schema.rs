//! Shared Diesel schema for crawler SQLite tables.

diesel::table! {
    crawler_keyword (id) {
        id -> Integer,
        batch_id -> Text,
        text -> Text,
        enabled -> Integer,
    }
}

diesel::table! {
    crawler_channel (id) {
        id -> Integer,
        job_id -> Text,
        keyword -> Text,
        platform -> Text,
        channel_id -> Text,
        title -> Text,
        country -> Nullable<Text>,
        subscriber_count -> Nullable<BigInt>,
        email -> Nullable<Text>,
        description -> Nullable<Text>,
        custom_url -> Nullable<Text>,
        email_status -> Text,
        enrich_attempts -> Integer,
        enrich_error -> Nullable<Text>,
        enriched_at -> Nullable<Text>,
        verified_email -> Nullable<Text>,
    }
}

diesel::table! {
    crawler_setting (key) {
        key -> Text,
        value -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(crawler_keyword, crawler_channel, crawler_setting);
