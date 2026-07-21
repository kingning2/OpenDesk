//! Diesel schema for `opendesk.db` tables.

diesel::table! {
    background_job (id) {
        id -> Text,
        job_type -> Text,
        payload_json -> Text,
        status -> Text,
        progress -> Float,
        error_message -> Nullable<Text>,
        worker_pid -> Nullable<Integer>,
        created_at -> Text,
        started_at -> Nullable<Text>,
        completed_at -> Nullable<Text>,
    }
}

diesel::table! {
    customer (id) {
        id -> Text,
        display_name -> Nullable<Text>,
        email -> Text,
        whatsapp_phone -> Nullable<Text>,
        source_channel -> Text,
        source_meta -> Nullable<Text>,
        lifecycle_status -> Text,
        outreach_stage -> Text,
        quoted_price -> Nullable<Float>,
        quoted_currency -> Nullable<Text>,
        quoted_at -> Nullable<Text>,
        pricing_tier -> Nullable<Text>,
        cooperation_status -> Text,
        package_name -> Nullable<Text>,
        monthly_fee -> Nullable<Float>,
        contract_start -> Nullable<Text>,
        contract_end -> Nullable<Text>,
        notes -> Nullable<Text>,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    quote_history (id) {
        id -> Text,
        customer_id -> Text,
        old_price -> Nullable<Float>,
        new_price -> Nullable<Float>,
        currency -> Nullable<Text>,
        old_tier -> Nullable<Text>,
        new_tier -> Nullable<Text>,
        reason -> Nullable<Text>,
        changed_by -> Nullable<Text>,
        created_at -> Text,
    }
}

diesel::table! {
    customer_timeline (id) {
        id -> Text,
        customer_id -> Text,
        entry_type -> Text,
        ref_id -> Nullable<Text>,
        summary -> Text,
        metadata_json -> Nullable<Text>,
        created_at -> Text,
    }
}

diesel::table! {
    cooperation_audit (id) {
        id -> Text,
        customer_id -> Text,
        field_name -> Text,
        old_value -> Nullable<Text>,
        new_value -> Nullable<Text>,
        changed_by -> Nullable<Text>,
        created_at -> Text,
    }
}

diesel::joinable!(quote_history -> customer (customer_id));
diesel::joinable!(customer_timeline -> customer (customer_id));
diesel::joinable!(cooperation_audit -> customer (customer_id));

diesel::allow_tables_to_appear_in_same_query!(
    background_job,
    customer,
    quote_history,
    customer_timeline,
    cooperation_audit,
);
