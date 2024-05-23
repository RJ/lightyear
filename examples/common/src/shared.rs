use lightyear::prelude::{Mode, SharedConfig, TickConfig};
use std::time::Duration;

pub const FIXED_TIMESTEP_HZ: f64 = 64.0;

/// The [`SharedConfig`] must be shared between the `ClientConfig` and `ServerConfig`
pub fn shared_config(mode: Mode) -> SharedConfig {
    SharedConfig {
        client_send_interval: Duration::from_millis(0),
        // send an update every frame
        server_send_interval: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
        tick: TickConfig {
            tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
        },
        mode,
    }
}
