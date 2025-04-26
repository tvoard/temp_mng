use std::{sync::Once, time::Instant};

static mut SERVER_START_TIME: Option<Instant> = None;
static INIT: Once = Once::new();

pub fn initialize_server_start_time() {
    tracing::info!("Initializing server start time...");

    unsafe {
        INIT.call_once(|| {
            SERVER_START_TIME = Some(Instant::now());
        });
    }
}

pub fn get_server_runtime() -> u64 {
    unsafe {
        SERVER_START_TIME
            .map(|start_time| start_time.elapsed().as_secs())
            .unwrap_or(0)
    }
}
