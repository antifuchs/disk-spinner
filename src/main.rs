use clap::Parser;
use std::{path::PathBuf, str::FromStr};
use tracing::{info, warn};

#[derive(Debug, Clone)]
struct ValidDevice {
    path: PathBuf,
    partition: Option<u64>,
    device: block_utils::Device,
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

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the device to test.
    ///
    /// This should be a mechanical disk block device (e.g. /dev/sda,
    /// /dev/disk/by-id/wwn-...).
    #[clap(value_parser = clap::value_parser!(ValidDevice))]
    device: ValidDevice,

    /// Test the device even if the media type is not a spinning disk.
    #[clap(long)]
    allow_any_media: bool,

    /// Run the test even if the given path is a block device but not
    /// a disk (e.g. a single partition).
    #[clap(long)]
    allow_any_block_device: bool,
}

fn main() -> anyhow::Result<()> {
    tracing::subscriber::set_global_default(tracing_subscriber::FmtSubscriber::new())
        .expect("setting tracing default failed");
    let args = Args::parse();

    let ValidDevice {
        device,
        partition,
        path,
    } = args.device;
    // Sanity checks:
    if partition.is_some() {
        if !args.allow_any_block_device {
            anyhow::bail!("Device is not a whole disk but a partition - pass --allow-any-block-device to run tests anyway.");
        } else {
            warn!(?partition, "Testing a partition but running tests anyway.");
        }
    }
    if device.media_type != block_utils::MediaType::Rotational {
        if !args.allow_any_media {
            anyhow::bail!("Device is not a rotational disk - this tool may be harmful for solid-state drives! Pass --allow-any-media to run anyway.");
        } else {
            warn!(?device.media_type, "Media type is not as expected but running tests anyway.");
        }
    }
    info!(?partition, ?device, ?path);
    Ok(())
}
