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
        extra_json -> Nullable<Text>,
        source_ref -> Nullable<Text>,
    }
}

diesel::table! {
    mail_template (id) {
        id -> Text,
        name -> Text,
        template_intent -> Text,
        subject_template -> Text,
        body_text_template -> Text,
        body_html_template -> Nullable<Text>,
        locale -> Nullable<Text>,
        is_system -> Bool,
        is_active -> Bool,
        sort_order -> BigInt,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    mail_account (id) {
        id -> Text,
        label -> Text,
        from_address -> Text,
        from_name -> Nullable<Text>,
        smtp_host -> Text,
        smtp_port -> BigInt,
        use_tls -> Bool,
        username -> Text,
        password_ref -> Text,
        password_value -> Text,
        imap_host -> Nullable<Text>,
        imap_port -> Nullable<BigInt>,
        imap_use_tls -> Nullable<Bool>,
        imap_sync_enabled -> Bool,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    mail_message (id) {
        id -> Text,
        customer_id -> Nullable<Text>,
        template_id -> Nullable<Text>,
        account_id -> Nullable<Text>,
        status -> Text,
        direction -> Text,
        subject -> Text,
        body_text -> Text,
        body_html -> Nullable<Text>,
        error_message -> Nullable<Text>,
        sent_at -> Nullable<Text>,
        received_at -> Nullable<Text>,
        imap_uid -> Nullable<BigInt>,
        imap_folder -> Nullable<Text>,
        rfc_message_id -> Nullable<Text>,
        in_reply_to -> Nullable<Text>,
        references_header -> Nullable<Text>,
        is_favorite -> Bool,
        is_read -> Bool,
        open_tracking_id -> Nullable<Text>,
        opened_at -> Nullable<Text>,
        open_count -> BigInt,
        created_at -> Text,
        updated_at -> Text,
        to_address -> Nullable<Text>,
        from_address -> Nullable<Text>,
        source_ref -> Nullable<Text>,
    }
}

diesel::table! {
    mail_integration_setting (id) {
        id -> Text,
        enabled -> Bool,
        api_base -> Text,
        pixel_path_template -> Text,
        query_path_template -> Text,
        parse_script -> Text,
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

diesel::table! {
    script_snippet (id) {
        id -> Text,
        source_id -> Text,
        title -> Text,
        stage -> Nullable<Text>,
        trigger_text -> Nullable<Text>,
        description -> Nullable<Text>,
        from_stage -> Nullable<Text>,
        to_stage -> Nullable<Text>,
        tags_json -> Text,
        body_text -> Text,
        category_l1 -> Nullable<Text>,
        category_l2 -> Nullable<Text>,
        needs_boss_input -> Bool,
        boss_input_hint -> Nullable<Text>,
        sort_order -> BigInt,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    llm_setting (id) {
        id -> Text,
        provider -> Text,
        base_url -> Nullable<Text>,
        model_id -> Text,
        api_key_ref -> Text,
        has_api_key -> Bool,
        updated_at -> Text,
    }
}

diesel::table! {
    mail_imap_sync_state (account_id, folder) {
        account_id -> Text,
        folder -> Text,
        uidvalidity -> BigInt,
        highest_modseq -> Text,
        last_uid -> BigInt,
        last_sync_at -> Nullable<Text>,
        full_synced -> Bool,
        last_error -> Nullable<Text>,
    }
}

diesel::table! {
    mail_pending_send (id) {
        id -> Text,
        account_id -> Nullable<Text>,
        recipients_json -> Text,
        subject -> Text,
        body_text -> Text,
        body_html -> Nullable<Text>,
        status -> Text,
        scheduled_at -> Nullable<Text>,
        source_ref -> Nullable<Text>,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    legacy_json_doc (id) {
        id -> Text,
        kind -> Text,
        payload_json -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    wf_rt_instance (instance_id) {
        instance_id -> Text,
        definition_id -> Nullable<Text>,
        definition_json -> Text,
        state -> Text,
        context_json -> Text,
        context_version -> BigInt,
        error_code -> Nullable<Text>,
        error_message -> Nullable<Text>,
        heartbeat_at -> Nullable<Text>,
        created_at -> Text,
        updated_at -> Text,
        started_at -> Nullable<Text>,
        finished_at -> Nullable<Text>,
    }
}

diesel::table! {
    wf_rt_node_instance (instance_id, node_id) {
        instance_id -> Text,
        node_id -> Text,
        node_type -> Text,
        state -> Text,
        attempt -> BigInt,
        max_retry -> BigInt,
        retry_state_json -> Text,
        input_json -> Nullable<Text>,
        output_json -> Nullable<Text>,
        error_message -> Nullable<Text>,
        started_at -> Nullable<Text>,
        finished_at -> Nullable<Text>,
        duration_ms -> Nullable<BigInt>,
    }
}

diesel::table! {
    wf_rt_log (id) {
        id -> Text,
        instance_id -> Text,
        node_id -> Nullable<Text>,
        level -> Text,
        event_kind -> Text,
        payload_json -> Text,
        created_at -> Text,
    }
}

diesel::joinable!(quote_history -> customer (customer_id));
diesel::joinable!(customer_timeline -> customer (customer_id));
diesel::joinable!(cooperation_audit -> customer (customer_id));
diesel::joinable!(mail_message -> customer (customer_id));
diesel::joinable!(mail_message -> mail_account (account_id));
diesel::joinable!(mail_message -> mail_template (template_id));
diesel::joinable!(mail_imap_sync_state -> mail_account (account_id));
diesel::joinable!(mail_pending_send -> mail_account (account_id));
diesel::joinable!(wf_rt_node_instance -> wf_rt_instance (instance_id));
diesel::joinable!(wf_rt_log -> wf_rt_instance (instance_id));

diesel::allow_tables_to_appear_in_same_query!(
    background_job,
    customer,
    mail_template,
    mail_account,
    mail_message,
    mail_integration_setting,
    quote_history,
    customer_timeline,
    cooperation_audit,
    script_snippet,
    llm_setting,
    mail_imap_sync_state,
    mail_pending_send,
    legacy_json_doc,
    wf_rt_instance,
    wf_rt_node_instance,
    wf_rt_log,
);
