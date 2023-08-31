// @generated automatically by Diesel CLI.

diesel::table! {
    configs (id) {
        id -> Int8,
        compatibility -> Nullable<Varchar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        subject_id -> Nullable<Int8>,
    }
}

diesel::table! {
    schema_versions (id) {
        id -> Int8,
        version -> Nullable<Int4>,
        subject_id -> Int8,
        schema_id -> Int8,
    }
}

diesel::table! {
    schemas (id) {
        id -> Int8,
        fingerprint -> Varchar,
        json -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        fingerprint2 -> Nullable<Varchar>,
    }
}

diesel::table! {
    subjects (id) {
        id -> Int8,
        name -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    configs,
    schema_versions,
    schemas,
    subjects,
);
