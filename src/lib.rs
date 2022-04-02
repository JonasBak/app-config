pub use config_derive::*;
pub use config_simple;

pub trait AppConfig {
    type Builder;
    fn builder() -> Self::Builder;
}

#[cfg(test)]
mod tests {
}
