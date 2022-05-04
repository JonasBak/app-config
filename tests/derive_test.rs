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

#[derive(AppConfig, Debug, PartialEq)]
struct DoubleNestingConfig {
    #[nested_field]
    nested_b: NestingConfig,
}

#[allow(dead_code)]
#[derive(AppConfig, Debug, PartialEq)]
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

#[allow(dead_code)]
#[derive(AppConfig)]
#[builder_derive(Deserialize)]
struct NestedDeserializeConfig {
    #[nested_field]
    nested: DeserializeConfig,
}

#[derive(AppConfig, Debug, PartialEq)]
struct OptionalNestedConfig {
    #[nested_field]
    optional: Option<BasicConfig>,
}

#[derive(AppConfig, Debug, PartialEq)]
struct MultipleOptionalConfig {
    optional_a: Option<usize>,
    optional_b: Option<usize>,
    #[nested_field]
    optional: Option<BasicConfig>,
}

#[derive(AppConfig, Debug, PartialEq)]
struct OptionalNestedWithDefaultConfig {
    #[nested_field]
    optional: Option<AttrDefaultConfig>,
}

#[derive(AppConfig, Debug, PartialEq)]
enum EnumConfig {
    ChoiceA(BasicConfig),
    ChoiceB(AttrDefaultConfig),
}

#[derive(AppConfig, Debug, PartialEq)]
struct NestedEnumConfig {
    #[nested_field]
    nested: EnumConfig,
}

#[derive(AppConfig, Debug, PartialEq)]
#[builder_derive(Deserialize)]
enum EnumDeserializeConfig {
    ChoiceA(DeserializeConfig),
}

#[derive(AppConfig)]
#[builder_derive(Deserialize)]
struct NestedEnumDeserializeConfig {
    #[nested_field]
    nested: EnumDeserializeConfig,
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
    std::env::set_var("CONFIG_FIELD_A", "test a");
    std::env::set_var("CONFIG_FIELD_B", "123");
    std::env::set_var("CONFIG_FIELD_C", "false");
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
    std::env::set_var("CONFIG_FROM_ENV_ERR_FIELD_A", "test a");
    std::env::set_var("CONFIG_FROM_ENV_ERR_FIELD_B", "test b");
    std::env::set_var("CONFIG_FROM_ENV_ERR_FIELD_C", "test c");
    let result = MultipleTypesConfig::builder().from_env_prefixed("CONFIG_from_env_err");
    assert!(result.is_err());
    let errors = result.err().unwrap().len();
    assert_eq!(errors, 2);
}

#[test]
fn from_env_custom_prefix() {
    std::env::set_var("MY_CUSTOM_PREFIX_FIELD_A", "test a");
    std::env::set_var("MY_CUSTOM_PREFIX_FIELD_B", "123");
    std::env::set_var("MY_CUSTOM_PREFIX_FIELD_C", "false");
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
    let builder = AttrDefaultConfig::builder().default();
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
fn double_nested() {
    let result = DoubleNestingConfig::builder()
        .nested_b(
            NestingConfig::builder().nested_a(
                BasicConfig::builder()
                    .field_a("test a".into())
                    .field_b("test b".into())
                    .field_c("test c".into()),
            ),
        )
        .try_build();
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.nested_b.nested_a.field_a, "test a");
    assert_eq!(config.nested_b.nested_a.field_b, "test b");
    assert_eq!(config.nested_b.nested_a.field_c, "test c");
}

#[test]
fn nested_from_env() {
    std::env::set_var("CONFIG_NESTED_FROM_ENV_NESTED_A_FIELD_A", "test a");
    std::env::set_var("CONFIG_NESTED_FROM_ENV_NESTED_A_FIELD_B", "test b");
    std::env::set_var("CONFIG_NESTED_FROM_ENV_NESTED_A_FIELD_C", "test c");
    let builder_result = NestingConfig::builder().from_env_prefixed("CONFIG_nested_from_env");
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

#[test]
fn nested_deserialize_builder() {
    let config_yml = "nested:\n  field_a: test a\n  field_b: test b";
    let builder: <NestedDeserializeConfig as AppConfig>::Builder =
        serde_yaml::from_str(&config_yml).unwrap();
    assert_eq!(builder.nested.field_a, Some("test a".into()));
    assert_eq!(builder.nested.field_b, Some("test b".into()));
    assert_eq!(builder.nested.field_c, None);
}

#[test]
fn map_nested() {
    let result = NestingConfig::builder()
        .map_nested_a(|b| {
            b.field_a("test a".into())
                .field_b("test b".into())
                .field_c("test c".into())
        })
        .try_build();
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.nested_a.field_a, "test a");
    assert_eq!(config.nested_a.field_b, "test b");
    assert_eq!(config.nested_a.field_c, "test c");
}

#[test]
fn nested_combine() {
    let result = NestingConfig::builder()
        .map_nested_a(|b| b.field_a("test a".into()).field_b("test b".into()))
        .combine(NestingConfig::builder().map_nested_a(|b| b.field_c("test c".into())))
        .try_build();
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.nested_a.field_a, "test a");
    assert_eq!(config.nested_a.field_b, "test b");
    assert_eq!(config.nested_a.field_c, "test c");

    let result = NestingConfig::builder()
        .map_nested_a(|b| b.field_a("test a".into()).field_b("test b".into()))
        .combine(NestingConfig::builder().map_nested_a(|b| {
            b.field_b("should not be used".into())
                .field_c("test c".into())
        }))
        .try_build();
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.nested_a.field_a, "test a");
    assert_eq!(config.nested_a.field_b, "test b");
    assert_eq!(config.nested_a.field_c, "test c");
}

#[test]
fn default_attr_not_used_when_from_env() {
    std::env::set_var("NO_DEFAULT_VALUE_FIELD_C", "false");
    let builder = AttrDefaultConfig::builder()
        .from_env_prefixed("NO_DEFAULT_VALUE")
        .unwrap();
    assert_eq!(builder.field_a, None);
    assert_eq!(builder.field_b, None);
    assert_eq!(builder.field_c, Some(false));
}

#[test]
fn optional_nested_field_none() {
    let result = OptionalNestedConfig::builder().try_build();
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.optional, None);
}

#[test]
fn optional_nested_field_partial_fails() {
    let result = OptionalNestedConfig::builder()
        .map_optional(|b| b.field_a("test a".into()).field_b("test b".into()))
        .try_build();
    assert!(result.is_err());

    let result = OptionalNestedConfig::builder()
        .map_optional(|b| b.field_a("test a".into()).field_b("test b".into()))
        .combine(
            OptionalNestedConfig::builder()
                .map_optional(|b| b.field_a("test a".into()).field_b("test b".into())),
        )
        .try_build();
    assert!(result.is_err());
}

#[test]
fn optional_nested_field_some() {
    let result = OptionalNestedConfig::builder()
        .map_optional(|b| {
            b.field_a("test a".into())
                .field_b("test b".into())
                .field_c("test c".into())
        })
        .try_build();
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(
        config.optional,
        Some(BasicConfig {
            field_a: "test a".into(),
            field_b: "test b".into(),
            field_c: "test c".into()
        })
    );

    let result = OptionalNestedConfig::builder()
        .map_optional(|b| b.field_a("test a".into()).field_b("test b".into()))
        .combine(
            OptionalNestedConfig::builder()
                .map_optional(|b| b.field_a("ignored".into()).field_c("test c".into())),
        )
        .try_build();
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(
        config.optional,
        Some(BasicConfig {
            field_a: "test a".into(),
            field_b: "test b".into(),
            field_c: "test c".into()
        })
    );
}

#[test]
fn builder_empty() {
    let builder = BasicConfig::builder();
    assert_eq!(builder.is_empty(), true);

    let builder = BasicConfig::builder().field_a("test a".into());
    assert_eq!(builder.is_empty(), false);
}

#[test]
fn optional_nested_ignore_defaults() {
    let result = OptionalNestedWithDefaultConfig::builder().try_build();
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.optional, None);
}

#[test]
fn enum_config() {
    let mut builder = EnumConfig::builder();
    builder.choice_b = AttrDefaultConfig::builder().default();
    builder.using = Some("choice_b".into());
    let result = builder.try_build();
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(
        config,
        EnumConfig::ChoiceB(AttrDefaultConfig::builder().default().try_build().unwrap())
    );
}

#[test]
fn enum_defaults() {
    let result = EnumConfig::builder().default().using_choice_b().try_build();
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(
        config,
        EnumConfig::ChoiceB(AttrDefaultConfig::builder().default().try_build().unwrap())
    );
}

#[test]
fn enum_combine() {
    let result = EnumConfig::builder()
        .map_choice_a(|b| b.field_a("test a".into()).field_b("test b".into()))
        .combine(
            EnumConfig::builder()
                .map_choice_a(|b| b.field_b("ignored".into()).field_c("test c".into()))
                .using_choice_a(),
        )
        .combine(EnumConfig::builder().using_choice_b())
        .try_build();
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(
        config,
        EnumConfig::ChoiceA(
            BasicConfig::builder()
                .field_a("test a".into())
                .field_b("test b".into())
                .field_c("test c".into())
                .try_build()
                .unwrap()
        )
    );
}

#[test]
fn enum_from_env() {
    std::env::set_var("CONFIG_ENUM_FROM_ENV_USING", "choice_a");
    std::env::set_var("CONFIG_ENUM_FROM_ENV_CHOICE_A_FIELD_C", "test c");
    let config = EnumConfig::builder()
        .from_env_prefixed("CONFIG_enum_from_env")
        .unwrap()
        .map_choice_a(|b| b.field_a("test a".into()).field_b("test b".into()))
        .try_build()
        .unwrap();
    assert_eq!(
        config,
        EnumConfig::ChoiceA(
            BasicConfig::builder()
                .field_a("test a".into())
                .field_b("test b".into())
                .field_c("test c".into())
                .try_build()
                .unwrap()
        )
    );
}

#[test]
fn nested_enum() {
    let config = NestedEnumConfig::builder()
        .map_nested(|b| b.default().using_choice_b())
        .try_build()
        .unwrap();
    assert_eq!(
        config.nested,
        EnumConfig::ChoiceB(AttrDefaultConfig::builder().default().try_build().unwrap()),
    );
}

#[test]
fn nested_deserialize_enum() {
    let config_yml = "nested: {using: choice_a, choice_a: {field_a: 'test a', field_b: 'test b', field_c: 'test c'}}";
    let builder: <NestedEnumDeserializeConfig as AppConfig>::Builder =
        serde_yaml::from_str(&config_yml).unwrap();
    let config = builder.try_build().unwrap();
    assert_eq!(
        config.nested,
        EnumDeserializeConfig::ChoiceA(
            DeserializeConfig::builder()
                .field_a("test a".into())
                .field_b("test b".into())
                .field_c("test c".into())
                .try_build()
                .unwrap()
        ),
    );
}
