use std::env;

pub fn get_port() -> String {
    env::var("PORT").unwrap_or("22000".to_string())
}

pub fn get_host() -> String {
    env::var("HOST").unwrap_or("127.0.0.1".to_string())
}
