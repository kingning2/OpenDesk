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
