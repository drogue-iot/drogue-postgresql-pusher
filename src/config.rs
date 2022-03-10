use serde::Deserialize;

pub trait ConfigFromEnv<'de>: Sized + Deserialize<'de> {
    fn from_env() -> Result<Self, config::ConfigError> {
        Self::from(config::Environment::default())
    }

    fn from_env_prefix<S: AsRef<str>>(prefix: S) -> Result<Self, config::ConfigError> {
        Self::from(config::Environment::with_prefix(prefix.as_ref()))
    }

    fn from(env: config::Environment) -> Result<Self, config::ConfigError>;
}

impl<'de, T: Deserialize<'de> + Sized> ConfigFromEnv<'de> for T {
    fn from(env: config::Environment) -> Result<T, config::ConfigError> {
        let cfg = config::Config::builder()
            .add_source(env.separator("__"))
            .build()?;
        cfg.try_deserialize()
    }
}
