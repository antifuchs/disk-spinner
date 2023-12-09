extern crate block_utils;
use std::{path::PathBuf, str::FromStr};

#[derive(Debug, Clone)]
pub(crate) struct ValidDevice {
    pub path: PathBuf,
    pub partition: Option<u64>,
    pub device: block_utils::Device,
}

impl FromStr for ValidDevice {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (partition, device) = block_utils::get_device_from_path(s)?;
        Ok(Self {
            path: PathBuf::from(s),
            partition,
            device: device.ok_or(anyhow::anyhow!(
                "The device under test must be a valid block device."
            ))?,
        })
    }
}
