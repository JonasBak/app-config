pub use config_derive::*;

pub trait AppConfig {
    type Builder;
    fn builder() -> Self::Builder;
}

pub trait AppConfigChoice {
    type Choices;
}

#[cfg(test)]
mod tests {}
