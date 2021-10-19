// Show 'em your OS!
// By doncato.

// Imports
use chrono::Utc;
use clap::{App, Arg};
use discord_rpc_client::Client as RPC;
use log::*;
use std::{
    convert::TryFrom,
    thread,
    time::{Duration, SystemTime},
};
use sysinfo::{ComponentExt, ProcessorExt, System, SystemExt};

// Main function
fn main() {
    info!("Started");

    // Get command line parameters
    let options = App::new("SEYOS - Show 'em your OS")
        .version("1.1")
        .author("doncato <don.cato.dc11@gmail.com>")
        .about("Display your Operating System Information in a Discord Rich Presence")
        .arg(
            Arg::with_name("short-os-name")
                .short("s")
                .long("short-os-name")
                .help("Uses the short OS name in the presence information (may not make a difference depending on your OS)")
        )
        .arg(
            Arg::with_name("include-kernel")
                .short("k")
                .long("include-kernel")
                .help("Include the kernel version with the OS name")
        )
        .arg(
            Arg::with_name("additional-information")
                .short("a")
                .long("additional-information")
                .takes_value(true)
                .help("Set which additional information about your system should be displayed (use -l to get a list)")
        )
        .arg(
            Arg::with_name("list-available-information")
                .short("l")
                .long("list-additional-information")
                .overrides_with_all(&["short-os-name", "include-kernel", "additional-information", "application-time"])
                .help("List additional information available to be displayed")
        )
        .arg(
            Arg::with_name("application-time")
                .short("t")
                .long("application-time")
                .help("Display the time since the Application was started instead of system uptime")
        )
        .get_matches();

    // If requested, list all possibilities of infos
    if options.is_present("list-available-information") {
        print!("{}", AvailableInfos::get_all());
        return;
    }

    // Set the System Instance
    let mut sys = System::new_all();
    sys.refresh_system();
    let platform = get_os(&sys).1;
    //let mac = "mac".to_string();
    // Set the Discord Client instance
    let mut rpc = RPC::new(match platform.as_ref() {
        "darwin" => 899912704188379136,
        "windows" => 0,
        _ => 898584015076982865,
    });
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
        infos.os_name = if options.is_present("short-os-name") {
            os.0
        } else {
            sys.long_os_version().unwrap_or(os.0)
        };
        if options.is_present("include-kernel") {
            infos.os_name =
                infos.os_name + " " + &sys.kernel_version().unwrap_or("0.0".to_string());
        }
        infos.asset_name = os.1;
        if !options.is_present("application-time") {
            infos.uptime = sys.boot_time()
        };
        infos.information = if !options.is_present("additional-information") {
            format!("Load: {}", sys.load_average().five)
        } else {
            parse_infos(options.value_of("additional-information").unwrap_or("load"))
                .get_requested(&sys)
        };

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
        .replace("os", "")
        .trim()
        .to_string();

    return (name, identifier);
}

fn parse_infos(input: &str) -> AvailableInfos {
    let i = input.to_lowercase();
    let r = match i.trim() {
        "hostname" => AvailableInfos::Hostname,
        "average-temperature" => AvailableInfos::AvgTemperature,
        "memory" => AvailableInfos::Memory,
        "cpu" => AvailableInfos::Cpu,
        _ => AvailableInfos::Load,
    };
    return r;
}

enum AvailableInfos {
    Hostname,
    AvgTemperature,
    Memory,
    Cpu,
    Load,
}
impl AvailableInfos {
    fn get_all() -> String {
        "Hostname\nAverage-Temperature\nMemory\nCpu\n".to_string()
    }
    fn get_requested(self, system: &System) -> String {
        match self {
            AvailableInfos::Hostname => system
                .host_name()
                .unwrap_or(system.name().unwrap_or("".to_string())),
            AvailableInfos::AvgTemperature => {
                let mut avg: f32 = 0.0;
                let length: f32 = system.components().len() as f32;
                for i in system.components().iter() {
                    avg = avg + i.temperature();
                }
                format!("{0:.2} °C", avg / length)
            }
            AvailableInfos::Memory => format!(
                "{0:.2}/{1:.2} GB RAM",
                (system.available_memory() / 1_000_000),
                (system.total_memory() / 1_000_000)
            ),
            AvailableInfos::Cpu => {
                let cpu = system.global_processor_info();
                format!("{0} ({1:.2}%)", cpu.brand(), cpu.cpu_usage())
            }
            AvailableInfos::Load => format!("Load: {}", system.load_average().five),
        }
    }
}

struct PresenceInfo {
    os_name: String,
    information: String,
    asset_name: String,
    uptime: u64,
}
impl PresenceInfo {
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
                .timestamps(|time| time.start(self.uptime))
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
