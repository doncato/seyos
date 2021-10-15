// Show 'em your OS!
// By doncato.

// Imports
use chrono::Utc;
use discord_rpc_client::Client as RPC;
use log::*;
use std::{
    convert::TryFrom,
    thread,
    time::{Duration, SystemTime},
};
use sysinfo::{System, SystemExt};

// Main function
fn main() {
    info!("Started");
    // Set the System Instance
    let mut sys = System::new_all();
    // Set the Discord Client instance
    let mut rpc = RPC::new(898584015076982865);
    let refresh_time = 20;
    let refresh_interval = Duration::from_secs(refresh_time);

    // Start the Client instance
    rpc.start();
    debug!("Started the Discord Client Instance");

    // Presence Information
    let mut infos = PresenceInfo::empty();

    // Start the main loop
    loop {
        let start = SystemTime::now();

        debug!("Started the main event loop");
        sys.refresh_system();
        let os = get_os(&sys);
        infos.os_name = sys.long_os_version().unwrap_or(os.0);
        infos.asset_name = os.1;
        infos.uptime = sys.uptime();
        infos.information = format!("Load: {}", sys.load_average().five);

        let set = infos.set(rpc);
        rpc = set.0;
        if !set.1 {
            error!(
                "Couldn't set the Discord Rich Presence! Retrying in about {} seconds",
                refresh_time
            )
        }

        thread::sleep(
            refresh_interval
                - (SystemTime::now().duration_since(start)).unwrap_or(Duration::from_secs(0)),
        );
    }
}

/// A function to get the os name, returns a
/// tuple of the name, and a lowercase identifier (e.g. 'arch'
/// or 'ubuntu'). If the detection fails ("Linux", "default")
/// are returned.
fn get_os(system: &System) -> (String, String) {
    let sys_name = system.name();
    let name = sys_name.clone().unwrap_or("Linux".to_string());
    let identifier = sys_name
        .unwrap_or("default".to_string())
        .to_string()
        .to_lowercase()
        .replace("linux", "")
        .trim()
        .to_string();

    return (name, identifier);
}

struct PresenceInfo {
    os_name: String,
    information: String,
    asset_name: String,
    uptime: u64,
}
impl PresenceInfo {
    /// Create a new Presence Info
    fn new(os_name: String, information: String, asset_name: String, uptime: u64) -> Self {
        PresenceInfo {
            os_name,
            information,
            asset_name,
            uptime,
        }
    }
    /// Create an empty Presence Info
    fn empty() -> Self {
        PresenceInfo {
            os_name: "linux".to_string(),
            information: "".to_string(),
            asset_name: "default".to_string(),
            uptime: u64::try_from(Utc::now().timestamp()).expect("Time went backwards."),
        }
    }

    /// Set the Presence Info
    fn set(&self, mut discord_rpc: RPC) -> (RPC, bool) {
        match discord_rpc.set_activity(|a| {
            a.state(&self.information)
                .details(&self.os_name)
                .assets(|ass| {
                    ass.large_image(&self.asset_name)
                        .large_text(&self.asset_name)
                })
        }) {
            Ok(_) => (discord_rpc, true),
            Err(_) => (discord_rpc, false),
        }
    }
}
