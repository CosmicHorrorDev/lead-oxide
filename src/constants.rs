use std::time::Duration;

// Note: pubproxy doesn't support https
pub const API_URI: &str = "http://pubproxy.com/api/proxy?";
pub const REPO_URI: &str = "https://github.com/LovecraftianHorror/lead-oxide";

// Yes the API says 1 second delay, but I was still occasionally getting rate limited,
// and 1.05 sec was also causing problems, so 1.1 sec is the new delay.
pub const DELAY: Duration = Duration::from_millis(1_100);
