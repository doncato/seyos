// Show 'em your OS!
// By doncato.

// Imports

use chrono::Utc;
use discord_rpc_client::Client as RPC;
use log::*;
use std::{convert::TryFrom, thread, time::Duration};
use sysinfo::{System, SystemExt};

fn main() {
    // Set the System Instance
    let mut sys = System::new_all();
    // Set the Discord Client instance
    let mut rpc = RPC::new(898584015076982865);
    let refresh_interval = Duration::from_secs(20);

    // Start the Client instance
    rpc.start();

    // Start the main loop
    loop {
        sys.refresh_all();

        println!("{:?}", get_os(&sys));

        thread::sleep(refresh_interval);
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
    asset_text: String,
    uptime: u32,
}
impl PresenceInfo {
    /// Create a new Presence Info
    fn new(
        os_name: String,
        information: String,
        asset_name: String,
        asset_text: String,
        uptime: u32,
    ) -> Self {
        PresenceInfo {
            os_name,
            information,
            asset_name,
            asset_text,
            uptime,
        }
    }
    /// Create an empty Presence Info
    fn empty() -> Self {
        PresenceInfo {
            os_name: "linux".to_string(),
            information: "".to_string(),
            asset_name: "default".to_string(),
            asset_text: "linux".to_string(),
            uptime: u32::try_from(Utc::now().timestamp()).expect("Time went backwards"),
        }
    }

    /// Set the Presence Info
    fn set(self, mut discord_rpc: RPC) -> Result<(), ()> {
        match discord_rpc.set_activity(|a| {
            a.state(&self.information)
                .details(&self.os_name)
                .assets(|ass| ass.large_image(self.asset_name).large_text(self.asset_text))
        }) {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }
}
