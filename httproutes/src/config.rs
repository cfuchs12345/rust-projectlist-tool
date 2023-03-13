use config::Config;

pub fn get_config() -> Config {

    Config::builder()
            .add_source(config::Environment::default())
            .build()
            .expect("Could not load config from env properties") // ok to panic, if the config cannot be loaded
}