pub use config_derive::*;

pub trait AppConfig {
    type Builder;
    fn builder() -> Self::Builder;
}

#[cfg(test)]
mod tests {}
