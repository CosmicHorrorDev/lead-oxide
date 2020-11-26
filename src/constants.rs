use std::time::Duration;

// Note: pubproxy doesn't support https
pub const API_URI: &str = "http://pubproxy.com/api/proxy?";
pub const REPO_URI: &str = "https://github.com/LovecraftianHorror/lead-oxide";

// Note: A shorter delay is used when testing
pub const DELAY: Duration = Duration::from_millis(if cfg!(test) { 100 } else { 1_100 });
