#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Anonymous,
    Elite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Protocol {
    Http,
    Socks4,
    Socks5,
}
