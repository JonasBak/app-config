use app_config::AppConfig;

#[derive(AppConfig)]
struct MyConfig {
    field_a: String,
    field_b: usize,
    #[config_field(default = true)]
    field_c: bool,
}

fn get_config() -> MyConfig {
    MyConfig::builder()
        .from_env()
        .unwrap()
        .field_a("foo".into())
        .try_build()
        .unwrap()
}

#[test]
fn readme_example() {
    // This test is a bit unpredictable
    std::env::set_var("CONFIG_field_b", "123");
    let config = get_config();
    assert_eq!(config.field_a, "foo");
    assert_eq!(config.field_b, 123);
    assert_eq!(config.field_c, true);
}
