use iso_country::Country;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Allow,
    Block,
}

impl Default for Action {
    fn default() -> Self {
        Self::Block
    }
}

// TODO: this could be valid for the whole time now, so could remove the builder for it
#[derive(Serialize, Clone, Debug, PartialEq)]
pub enum Countries {
    #[serde(rename = "country")]
    AllowList(String),
    #[serde(rename = "not_country")]
    BlockList(String),
}

impl Countries {
    #[must_use]
    pub fn allow() -> CountriesBuilder {
        CountriesBuilder::new(Action::Allow)
    }

    #[must_use]
    pub fn block() -> CountriesBuilder {
        CountriesBuilder::new(Action::Block)
    }

    #[must_use]
    pub fn allowlist(countries: &[Country]) -> Self {
        Self::allow().countries(countries).build()
    }

    #[must_use]
    pub fn blocklist(countries: &[Country]) -> Self {
        Self::block().countries(countries).build()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        match self {
            Self::AllowList(countries) => countries.is_empty(),
            Self::BlockList(countries) => countries.is_empty(),
        }
    }
}

impl Default for Countries {
    fn default() -> Self {
        CountriesBuilder::default().build()
    }
}

impl From<CountriesBuilder> for Countries {
    fn from(builder: CountriesBuilder) -> Self {
        let CountriesBuilder { list, action } = builder;

        match action {
            Action::Allow => Self::AllowList(list.join(",")),
            Action::Block => Self::BlockList(list.join(",")),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CountriesBuilder {
    list: Vec<String>,
    action: Action,
}

impl CountriesBuilder {
    #[must_use]
    fn new(action: Action) -> Self {
        Self {
            list: Vec::new(),
            action,
        }
    }

    #[must_use]
    pub fn country(mut self, country: Country) -> Self {
        // TODO: make sure this is documented. Mention that unknows are automatically filtered out
        // if any country is used in the allow or blocklist
        if let Country::Unspecified = country {
            panic!(format!(
                "This library doesn't allow `Unspecified` country in the allow or blocklist"
            ));
        }

        self.list.push(country.to_string());
        self
    }

    #[must_use]
    pub fn countries(mut self, countries: &[Country]) -> Self {
        for country in countries {
            self = self.country(*country);
        }

        self
    }

    #[must_use]
    pub fn build(self) -> Countries {
        Countries::from(self)
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
