// @generated automatically by Diesel CLI.

diesel::table! {
    acme_certificates (domain) {
        domain -> Nullable<Text>,
        cert_path -> Text,
        key_path -> Text,
        issuer_path -> Text,
        issued_at -> Integer,
        expires_at -> Integer,
        is_valid -> Integer,
    }
}

diesel::table! {
    audit_records (id) {
        id -> Nullable<Integer>,
        timestamp -> Integer,
        category -> Text,
        actor_id -> Text,
        device_id -> Text,
        surface -> Text,
        action -> Text,
        object -> Text,
        outcome -> Text,
        detail -> Text,
        ip_address -> Text,
    }
}

diesel::table! {
    authenticated_devices (device_id) {
        device_id -> Text,
        public_key -> Text,
        session_id -> Text,
        authenticated_at -> Integer,
        last_seen -> Integer,
    }
}

diesel::table! {
    crypt_keys (device_id) {
        device_id -> Nullable<Text>,
        private_key -> Binary,
        public_key -> Binary,
        created_at -> Integer,
    }
}

diesel::table! {
    dns_cache (domain, record_type, value) {
        domain -> Text,
        record_type -> Text,
        value -> Text,
        ttl -> Integer,
        timestamp -> Integer,
    }
}

diesel::table! {
    hosting (id) {
        id -> Nullable<Integer>,
        domain -> Text,
        status -> Text,
        domain_registrar -> Nullable<Text>,
        domain_registrar_token -> Nullable<Text>,
        domain_registered -> Integer,
        dns_provider -> Text,
        dns_provider_token -> Nullable<Text>,
        ns_addresses -> Nullable<Text>,
        dns_configured -> Integer,
        vm_provider -> Text,
        vm_provider_token -> Nullable<Text>,
        vm_instance_id -> Nullable<Text>,
        vm_ip_address -> Nullable<Text>,
        vm_ssh_user -> Nullable<Text>,
        vm_ssh_key_path -> Nullable<Text>,
        vm_created -> Integer,
        service_installed -> Integer,
        service_running -> Integer,
        created_at -> Integer,
        updated_at -> Integer,
        error_message -> Nullable<Text>,
    }
}

diesel::table! {
    nft_whitelist (ip) {
        ip -> Nullable<Text>,
        description -> Text,
        added_at -> Integer,
    }
}

diesel::table! {
    operation_logs (id) {
        id -> Nullable<Integer>,
        project_id -> Text,
        operation_type -> Text,
        external_system -> Text,
        status -> Text,
        started_at -> Integer,
        completed_at -> Nullable<Integer>,
        error_message -> Nullable<Text>,
        details -> Nullable<Text>,
    }
}

diesel::table! {
    sessions (session_id) {
        session_id -> Nullable<Text>,
        domain -> Text,
        session_type -> Text,
        connected_at -> Integer,
        last_seen -> Integer,
        request_count -> Integer,
        remote_addr -> Text,
    }
}

diesel::table! {
    sites (domain) {
        domain -> Nullable<Text>,
        public_key -> Text,
        status -> Text,
        last_seen -> Nullable<Integer>,
        created_at -> Integer,
        updated_at -> Integer,
    }
}

diesel::table! {
    webhook_allow_patterns (id) {
        id -> Nullable<Integer>,
        pattern -> Text,
        created_at -> Integer,
    }
}

diesel::table! {
    webhook_config (id) {
        id -> Nullable<Integer>,
        logging_enabled -> Integer,
    }
}

diesel::table! {
    webhook_requests (id) {
        id -> Nullable<Integer>,
        pattern -> Text,
        path -> Text,
        method -> Text,
        headers -> Text,
        body -> Text,
        remote_addr -> Text,
        received_at -> Integer,
    }
}

diesel::table! {
    wss_servers (domain) {
        domain -> Nullable<Text>,
        bind_addr -> Text,
        bind_port -> Integer,
        server_id -> Text,
        ping_interval -> Integer,
        idle_timeout -> Integer,
        max_connections -> Integer,
    }
}

diesel::table! {
    wss_sessions (session_id) {
        session_id -> Nullable<Text>,
        domain -> Text,
        connected_at -> Integer,
        last_seen -> Integer,
        message_count -> Integer,
        reconnect_count -> Integer,
    }
}

diesel::joinable!(wss_sessions -> wss_servers (domain));

diesel::allow_tables_to_appear_in_same_query!(
    acme_certificates,
    audit_records,
    authenticated_devices,
    crypt_keys,
    dns_cache,
    hosting,
    nft_whitelist,
    operation_logs,
    sessions,
    sites,
    webhook_allow_patterns,
    webhook_config,
    webhook_requests,
    wss_servers,
    wss_sessions,
);
