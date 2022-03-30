#![allow(dead_code)]

#[macro_export]
macro_rules! AppConfig {
    ( $( $x:ident: $t:ty = $v:expr ),*, ) => {
        use serde::{Deserialize};
        use std::str::FromStr;
        trait ParseFromEnv<T> {
            fn parse_from_env(var: String) -> Result<Option<T>, String>;
        }

        struct Env;

        impl<T: FromStr> ParseFromEnv<T> for Env {
            fn parse_from_env(var: String) -> Result<Option<T>, String> {
                match std::env::var(&var).map(|value| T::from_str(&value)) {
                    Ok(Ok(value)) => Ok(Some(value)),
                    Ok(Err(_)) => Err(format!("could not parse environment varaible {}", var)),
                    Err(std::env::VarError::NotPresent) => Ok(None),
                    _ => Err(format!("could not read environment varaible {}", var)),
                }
            }
        }

        #[derive(Debug, Clone)]
        pub struct AppConfig {
            $(
                pub $x: $t,
            )*
        }

        impl AppConfig {
        }

        #[derive(Deserialize, Debug)]
        pub struct AppConfigBuilder {
            $(
                pub $x: Option<$t>,
            )*
        }

        impl AppConfigBuilder {
            pub fn new() -> Self {
                AppConfigBuilder {
                    $(
                        $x: $v.into(),
                    )*
                }
            }
            $(
                pub fn $x(mut self, value: $t) -> Self {
                    self.$x = Some(value);
                    self
                }
            )*
            pub fn from_env() -> Result<Self, String> {
                let mut builder = Self::new();
                $(
                    builder.$x = <Env as ParseFromEnv::<$t>>::parse_from_env(format!("CONFIG_{}", stringify!($x)))?;
                )*
                Ok(builder)
            }
            pub fn merge(self, other: Self) -> Self {
                AppConfigBuilder {
                    $(
                        $x: self.$x.or(other.$x),
                    )*
                }
            }
            pub fn try_complete(self) -> Result<AppConfig, String> {
                Ok(AppConfig {
                    $(
                        $x: self.$x.ok_or_else(|| format!("field {} is required but not set", stringify!($x)))?,
                    )*
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    super::AppConfig! {
        config_file: String = "test.yml".to_string(),

        port: u16 = 3000,
        addr: std::net::IpAddr = "127.0.0.1".parse::<std::net::IpAddr>().unwrap(),

        postgres_username: String = None,
        postgres_password: String = None,
    }

    #[test]
    fn combined() {
        std::env::set_var("CONFIG_port", "3001");
        let app_config_env = AppConfigBuilder::from_env().unwrap();
        let app_config_builder = app_config_env.merge(AppConfigBuilder::new());
        let config_yml = "postgres_username: postgres\npostgres_password: password";
        let config_file_builder = serde_yaml::from_str(&config_yml).unwrap();
        let app_config = app_config_builder.merge(config_file_builder).try_complete().unwrap();
        assert_eq!(app_config.config_file, "test.yml");
        assert_eq!(app_config.port, 3001);
        assert_eq!(app_config.addr, "127.0.0.1".parse::<std::net::IpAddr>().unwrap());
        assert_eq!(app_config.postgres_username, "postgres");
        assert_eq!(app_config.postgres_password, "password");
    }
}
