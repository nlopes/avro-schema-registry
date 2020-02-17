use std::env;

pub fn get_schema_registry_password() -> String {
    env::var("SCHEMA_REGISTRY_PASSWORD").unwrap_or_else(|_| "test_password".to_string())
}
