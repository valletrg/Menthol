#[derive(Debug, Clone)]
pub struct Config {
    pub username: String,
    pub password: String,
    /// Server address (default: server.slsknet.org:2242 per spec)
    pub host: String,
    /// Server port (default: 2242 per spec)
    pub port: u32,
    /// Local listen port for incoming peer connections (default: 2234)
    pub listen_port: u32,
    /// Client major version - must be unique per spec to avoid impersonating other clients
    /// 182 is Menthol's assigned version (not 157=NS/Qt, 160=Nicotine+, 170=slskd)
    pub major_version: u32,
    /// Client minor version
    pub minor_version: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            host: "server.slsknet.org".to_string(),
            port: 2242, // Per LOGIN_FLOW.md spec
            listen_port: 2234,
            major_version: 182, // Unique client ID per spec
            minor_version: 1,
        }
    }
}
