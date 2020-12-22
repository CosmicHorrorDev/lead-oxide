use iso_country::Country;
use serde::{Deserialize, Serialize};

// TODO: this could be valid for the whole time now, so could remove the builder for it
#[derive(Serialize, Clone, Debug, PartialEq)]
pub enum Countries {
    #[serde(rename = "country")]
    AllowList(String),
    #[serde(rename = "not_country")]
    BlockList(String),
}

impl Countries {
    pub fn allow() -> Self {
        Self::AllowList(String::new())
    }

    pub fn block() -> Self {
        Self::BlockList(String::new())
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::AllowList(countries) => countries.is_empty(),
            Self::BlockList(countries) => countries.is_empty(),
        }
    }

    pub fn countries(mut self, countries: &[Country]) -> Self {
        for country in countries {
            self = self.country(*country);
        }

        self
    }

    pub fn country(self, country: Country) -> Self {
        // TODO: make sure this is documented. Mention that unknows are automatically filtered out
        // if any country is used in the allow or blocklist
        if let Country::Unspecified = country {
            panic!(format!(
                "This library doesn't allow `Unspecified` country in the allow or blocklist"
            ));
        }

        let push_country = |list: String, new_tag: Country| {
            let new_tag = new_tag.to_string();
            if list.is_empty() {
                new_tag
            } else {
                [list, new_tag].join(",")
            }
        };

        match self {
            Self::AllowList(list) => Self::AllowList(push_country(list, country)),
            Self::BlockList(list) => Self::BlockList(push_country(list, country)),
        }
    }
}

impl Default for Countries {
    fn default() -> Self {
        // Default is to block none
        Countries::block()
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Anonymous,
    Elite,
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Http,
    Socks4,
    Socks5,
}
