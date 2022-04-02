use app_config::AppConfig;
use serde::Deserialize;

#[derive(AppConfig, Debug, PartialEq)]
struct BasicConfig {
    field_a: String,
    field_b: String,
    field_c: String,
}
#[derive(AppConfig, Debug, PartialEq)]
struct MultipleTypesConfig {
    field_a: String,
    field_b: usize,
    field_c: bool,
}
#[derive(AppConfig, Debug, PartialEq)]
struct AttrDefaultConfig {
    #[config_field(default = "test default")]
    field_a: String,
    #[config_field(default = 321_usize)]
    field_b: usize,
    #[config_field(default = true)]
    field_c: bool,
}

fn _get_some_partial_builder() -> <AttrDefaultConfig as AppConfig>::Builder {
    AttrDefaultConfig::builder()
}

#[derive(AppConfig, Debug, PartialEq)]
struct NestingConfig {
    #[nested_field]
    nested_a: BasicConfig,
}

#[derive(AppConfig)]
#[builder_derive(Deserialize)]
struct DeserializeConfig {
    field_a: String,
    field_b: String,
    field_c: String,
}

#[derive(AppConfig, Debug, PartialEq)]
struct OptionalFieldConfig {
    optional: Option<usize>,
}

#[test]
fn set_builder_fields() {
    let builder = BasicConfig::builder()
        .field_a("test a".into())
        .field_b("test b".into());
    assert_eq!(builder.field_a, Some("test a".into()));
    assert_eq!(builder.field_b, Some("test b".into()));
    assert_eq!(builder.field_c, None);
}

#[test]
fn try_build_error() {
    let result = BasicConfig::builder().field_a("test a".into()).try_build();
    assert_eq!(result, Err(vec!["field_b", "field_c"]));
}

#[test]
fn try_build_ok() {
    let result = BasicConfig::builder()
        .field_a("test a".into())
        .field_b("test b".into())
        .field_c("test c".into())
        .try_build();
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.field_a, "test a");
    assert_eq!(config.field_b, "test b");
    assert_eq!(config.field_c, "test c");
}

#[test]
fn from_env_ok() {
    // This test is a bit unpredictable
    std::env::set_var("CONFIG_field_a", "test a");
    std::env::set_var("CONFIG_field_b", "123");
    std::env::set_var("CONFIG_field_c", "false");
    let builder_result = MultipleTypesConfig::builder().from_env();
    assert!(builder_result.is_ok());
    let config_result = builder_result.unwrap().try_build();
    assert!(config_result.is_ok());
    let config = config_result.unwrap();
    assert_eq!(config.field_a, "test a");
    assert_eq!(config.field_b, 123);
    assert_eq!(config.field_c, false);
}

#[test]
fn from_env_err() {
    // This test is a bit unpredictable
    std::env::set_var("CONFIG_field_a", "test a");
    std::env::set_var("CONFIG_field_b", "test b");
    std::env::set_var("CONFIG_field_c", "test c");
    let result = MultipleTypesConfig::builder().from_env();
    assert!(result.is_err());
    let errors = result.err().unwrap().len();
    assert_eq!(errors, 2);
}

#[test]
fn from_env_custom_prefix() {
    // This test is a bit unpredictable
    std::env::set_var("MY_CUSTOM_PREFIX_field_a", "test a");
    std::env::set_var("MY_CUSTOM_PREFIX_field_b", "123");
    std::env::set_var("MY_CUSTOM_PREFIX_field_c", "false");
    let builder_result = MultipleTypesConfig::builder().from_env_prefixed("MY_CUSTOM_PREFIX");
    assert!(builder_result.is_ok());
    let config_result = builder_result.unwrap().try_build();
    assert!(config_result.is_ok());
    let config = config_result.unwrap();
    assert_eq!(config.field_a, "test a");
    assert_eq!(config.field_b, 123);
    assert_eq!(config.field_c, false);
}

#[test]
fn combine_builders() {
    let builder = BasicConfig::builder()
        .field_a("test a".into())
        .field_b("test b".into())
        .combine(
            BasicConfig::builder()
                .field_a("ignored".into())
                .field_b("ignored".into())
                .field_c("test c".into()),
        );
    assert_eq!(builder.field_a, Some("test a".into()));
    assert_eq!(builder.field_b, Some("test b".into()));
    assert_eq!(builder.field_c, Some("test c".into()));
}

#[test]
fn default_attr() {
    let builder = AttrDefaultConfig::builder();
    assert_eq!(builder.field_a, Some("test default".into()));
    assert_eq!(builder.field_b, Some(321));
    assert_eq!(builder.field_c, Some(true));
}

#[test]
fn simple_nested() {
    let result = NestingConfig::builder()
        .nested_a(
            BasicConfig::builder()
                .field_a("test a".into())
                .field_b("test b".into())
                .field_c("test c".into()),
        )
        .try_build();
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.nested_a.field_a, "test a");
    assert_eq!(config.nested_a.field_b, "test b");
    assert_eq!(config.nested_a.field_c, "test c");
}

#[test]
fn nested_from_env() {
    // This test is a bit unpredictable
    std::env::set_var("CONFIG_nested_a_field_a", "test a");
    std::env::set_var("CONFIG_nested_a_field_b", "test b");
    std::env::set_var("CONFIG_nested_a_field_c", "test c");
    let builder_result = NestingConfig::builder().from_env();
    assert!(builder_result.is_ok());
    let config_result = builder_result.unwrap().try_build();
    assert!(config_result.is_ok());
    let config = config_result.unwrap();
    assert_eq!(config.nested_a.field_a, "test a");
    assert_eq!(config.nested_a.field_b, "test b");
    assert_eq!(config.nested_a.field_c, "test c");
}

#[test]
fn deserialize_builder() {
    let config_yml = "field_a: test a\nfield_b: test b";
    let builder: <DeserializeConfig as AppConfig>::Builder =
        serde_yaml::from_str(&config_yml).unwrap();
    assert_eq!(builder.field_a, Some("test a".into()));
    assert_eq!(builder.field_b, Some("test b".into()));
    assert_eq!(builder.field_c, None);
}

#[test]
fn optional_field_none() {
    let result = OptionalFieldConfig::builder().try_build();
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.optional, None);
}

#[test]
fn optional_field_some() {
    let result = OptionalFieldConfig::builder()
        .optional(Some(123))
        .try_build();
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.optional, Some(123));
}
