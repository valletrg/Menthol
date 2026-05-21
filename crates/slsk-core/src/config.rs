#[derive(Debug, Clone)]
pub struct Config {
    pub username:  String,
    pub password:  String,
    pub port:      u32,
    pub host:      String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            username:  String::new(),
            password:  String::new(),
            port:      2234,
            host:      "server.slsknet.org".to_string(),
        }
    }
}
