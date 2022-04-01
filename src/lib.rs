pub use config_simple;
pub use config_derive;


#[cfg(test)]
mod tests {
    use config_derive::AppConfig;
    #[derive(AppConfig, Debug, PartialEq)]
    struct AppConfig {
        field_a: String,
        field_b: String,
        field_c: String,
    }

    #[test]
    fn set_builder_fields() {
        let builder = AppConfig::builder()
            .field_a("test a".into())
            .field_b("test b".into());
        assert_eq!(builder.field_a, Some("test a".into()));
        assert_eq!(builder.field_b, Some("test b".into()));
        assert_eq!(builder.field_c, None);
    }

    #[test]
    fn try_build_error() {
        let result = AppConfig::builder()
            .field_a("test a".into()).try_build();
        assert_eq!(result, Err(vec!["field_b", "field_c"]));
    }

    #[test]
    fn try_build_ok() {
        let result = AppConfig::builder()
            .field_a("test a".into())
            .field_b("test b".into())
            .field_c("test c".into()).try_build();
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.field_a, "test a");
        assert_eq!(config.field_b, "test b");
        assert_eq!(config.field_c, "test c");
    }
}
