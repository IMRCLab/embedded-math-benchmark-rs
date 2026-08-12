//! Recovers a stuck RP2350 by poking its Rescue DP. See docs/hil-setup.md
//! "Recovering a stuck RP2350" for why/how. Usage: rp2350-rescue [probe-serial]

use probe_rs::{
    architecture::arm::{ApV2Address, FullyQualifiedApAddress, dp::DpAddress, sequences::DefaultArmSequence},
    probe::list::Lister,
};

const CTRL: u64 = 0;
const RESCUE_RESTART: u32 = 0x8000_0000;

fn main() -> anyhow::Result<()> {
    let serial = std::env::args().nth(1);

    let lister = Lister::new();
    let probes = lister.list_all();
    let probe_info = match &serial {
        Some(s) => probes
            .iter()
            .find(|p| p.serial_number.as_deref() == Some(s.as_str()))
            .unwrap_or_else(|| panic!("no probe with serial {s}; connected probes: {probes:#?}")),
        None => match probes.as_slice() {
            [p] => p,
            _ => panic!(
                "multiple probes connected, pass a serial as argv[1]: {probes:#?}"
            ),
        },
    };
    println!("Using probe: {probe_info}");

    let mut probe = probe_info.open()?;
    probe.attach_to_unspecified()?;
    let mut iface = probe
        .try_into_arm_debug_interface(DefaultArmSequence::create())
        .map_err(|(_, e)| e)?;

    iface.select_debug_port(DpAddress::Default)?;

    let rp_ap = FullyQualifiedApAddress::v2_with_dp(DpAddress::Default, ApV2Address(Some(0x80000)));

    let ctrl = iface.read_raw_ap_register(&rp_ap, CTRL)?;
    println!("RP-AP CTRL before: {ctrl:#010x}");

    iface.write_raw_ap_register(&rp_ap, CTRL, ctrl | RESCUE_RESTART)?;
    println!("wrote RESCUE_RESTART=1");

    let ctrl = iface.read_raw_ap_register(&rp_ap, CTRL)?;
    println!("RP-AP CTRL during: {ctrl:#010x}");

    iface.write_raw_ap_register(&rp_ap, CTRL, ctrl & !RESCUE_RESTART)?;
    println!("wrote RESCUE_RESTART=0");

    let ctrl = iface.read_raw_ap_register(&rp_ap, CTRL)?;
    println!("RP-AP CTRL after: {ctrl:#010x}");

    println!("Done. Now: probe-rs reset --chip RP235x, then probe-rs download --chip RP235x (may need a few retries).");

    Ok(())
}
