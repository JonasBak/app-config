# app-config

The goal of this package is to provide an easy way to build a "configuration struct" for your rust program, where you can combine data from multiple sources (cli arguments, environment, configuration file, defaults) to build a typed configuration object.

This is done by providing a trait your struct can `derive`. When you add `#[derive(AppConfig)]` to your struct, it will implement the `AppConfig` trait, creating a `Builder` type, and providing the `fn builder() -> Self::Builder` function. This `Builder` type can be used to specify fields with the "builder pattern", load available fields from the environment, load fields from a file, and combine the fields of multiple builders. The builder can be turned into your struct with the function `try_build()`, which fails if there are missing fields.

## Example

```rust
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
        .combine(MyConfig::builder().default())
        .try_build()
        .unwrap()
}
```

If we set the environment variable `CONFIG_FIELD_B=123`, and call `get_config` defined above, we should get an object that looks like this:

```rust
MyConfig {
    field_a: "foo",
    field_b: 123,
    field_c: true,
}
```

You can also use it in more complex cases, like below, with nested structs, deserialization, and combining multiple builders/sources. Earlier sources take precedent, so in this case the priority of the fields are:

1. `port` is always set to `PORT_NUMBER_FROM_CLI`
2. Then environment variables (like `COOL_APP_POSTGRES_PASSWORD=secret`)
3. Then values from the yaml file/string `CONFIG_YML`
4. Then default values

```rust
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
```

If we call `get_cool_app_config` with the environment variable `COOL_APP_POSTGRES_PASSWORD=secret` set, we should get something that looks like this:


```rust
CoolAppConfig {
    port: 80,
    addr: std::net::IpAddr::from([0, 0, 0, 0]),
    postgres: PostgresConfig {
        username: "postgres",
        password: "secret",
    },
    public_url: "example.com",
}
```
