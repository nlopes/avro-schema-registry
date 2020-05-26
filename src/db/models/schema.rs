table! {
    configs (id) {
        id -> Int8,
        compatibility -> Nullable<Varchar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        subject_id -> Nullable<Int8>,
    }
}

table! {
    schemas (id) {
        id -> Int8,
        fingerprint -> Varchar,
        json -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        fingerprint2 -> Nullable<Varchar>,
    }
}

table! {
    schema_versions (id) {
        id -> Int8,
        version -> Nullable<Int4>,
        subject_id -> Int8,
        schema_id -> Int8,
    }
}

table! {
    subjects (id) {
        id -> Int8,
        name -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

allow_tables_to_appear_in_same_query!(configs, schemas, schema_versions, subjects,);
