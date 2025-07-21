use bevy::prelude::*;

#[derive(Resource)]
pub struct ServerConfig {
    pub bind_port: u16,
}

impl ServerConfig {
    pub fn load() -> Result<Self> {
        let bind_port = std::env::args()
            .nth(1)
            .ok_or("Expected bind port as first argument")?;

        let bind_port = bind_port.parse().map_err(|_| "Invalid bind port format")?;

        Ok(ServerConfig { bind_port })
    }
}
