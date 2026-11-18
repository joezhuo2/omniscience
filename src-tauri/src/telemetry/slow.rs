use super::types::{BluetoothBattery, PhysicalDisk, PowerEvent};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SlowData {
    pub disks: Vec<PhysicalDisk>,
    pub bluetooth: Vec<BluetoothBattery>,
    pub metered: Option<bool>,
    pub power_events: Vec<PowerEvent>,
}

pub struct SlowProbe;

impl SlowProbe {
    pub fn start() -> Self {
        Self
    }

    pub fn latest(&self) -> SlowData {
        SlowData::default()
    }
}
