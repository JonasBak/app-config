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
        .try_build()
        .unwrap()
}
```

If we set the environment variable `CONFIG_field_b=123`, and call `get_config` defined above, we should get an object that looks like this:

```rust
MyConfig {
    field_a: "foo",
    field_b: 123,
    field_c: true,
}
```
