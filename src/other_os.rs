use std::{path::PathBuf, str::FromStr};

#[derive(Debug, Clone, Default)]
struct DeviceMetadata {
    pub physical_block_size: Option<u64>,
}

#[derive(Debug, Clone)]
pub(crate) struct ValidDevice {
    pub path: PathBuf,
    pub partition: Option<u64>,
    pub device: DeviceMetadata,
}

impl FromStr for ValidDevice {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            path: PathBuf::from(s),
            partition: None,
            device: DeviceMetadata::default(),
        })
    }
}
