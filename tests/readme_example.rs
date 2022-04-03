use app_config::AppConfig;
use serde::Deserialize;

#[derive(AppConfig)]
struct MyConfig {
    field_a: String,
    field_b: usize,
    #[config_field(default = true)]
    field_c: bool,
}

fn get_config() -> MyConfig {
    MyConfig::builder()
        .from_env_prefixed("CONFIG_readme_example")
        .unwrap()
        .field_a("foo".into())
        .combine(MyConfig::builder().default())
        .try_build()
        .unwrap()
}

#[derive(AppConfig)]
#[builder_derive(Deserialize)]
struct PostgresConfig {
    username: String,
    password: String,
}

#[derive(AppConfig)]
#[builder_derive(Deserialize)]
struct CoolAppConfig {
    #[config_field(default = 8080_u16)]
    port: u16,
    #[config_field(default = [127, 0, 0, 1])]
    addr: std::net::IpAddr,

    #[nested_field]
    postgres: PostgresConfig,

    #[config_field(default = "example.com")]
    public_url: String,
}

static CONFIG_YML: &'static str = r#"
addr: 0.0.0.0
postgres:
    username: postgres
    password: changeme
"#;

static PORT_NUMBER_FROM_CLI: u16 = 80;

fn get_cool_app_config() -> CoolAppConfig {
    CoolAppConfig::builder()
        .port(PORT_NUMBER_FROM_CLI)
        .combine(
            CoolAppConfig::builder()
                .from_env_prefixed("COOL_APP")
                .unwrap(),
        )
        .combine(serde_yaml::from_str(CONFIG_YML).unwrap())
        .combine(CoolAppConfig::builder().default())
        .try_build()
        .unwrap()
}

#[test]
fn readme_example1() {
    std::env::set_var("CONFIG_readme_example_field_b", "123");
    let config = get_config();
    assert_eq!(config.field_a, "foo");
    assert_eq!(config.field_b, 123);
    assert_eq!(config.field_c, true);
}

#[test]
fn readme_example2() {
    std::env::set_var("COOL_APP_port", "3000");
    std::env::set_var("COOL_APP_postgres_password", "secret");
    let config = get_cool_app_config();
    assert_eq!(config.port, 80);
    assert_eq!(config.addr, std::net::IpAddr::from([0, 0, 0, 0]));
    assert_eq!(config.postgres.username, "postgres");
    assert_eq!(config.postgres.password, "secret");
    assert_eq!(config.public_url, "example.com");
}
