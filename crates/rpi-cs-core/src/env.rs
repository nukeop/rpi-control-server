use std::env;

pub fn get(key: &str) -> String {
    env::var(key).expect(&format!("{} must be set", key))
}