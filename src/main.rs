use anyhow::Context;
use clap::Parser;
use indicatif::ProgressStyle;
use rand::prelude::*;
use rayon::prelude::*;
use std::{path::PathBuf, str::FromStr};
use tracing::{info, warn};
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[macro_use]
extern crate lazy_static;

mod crypto;
mod read_test;
mod write_test;

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
    /// Name of the devices to test.
    ///
    /// Each should be a mechanical disk block device (e.g. /dev/sda,
    /// /dev/disk/by-id/wwn-...).
    #[clap(value_parser = clap::value_parser!(ValidDevice), num_args = 1..)]
    devices: Vec<ValidDevice>,

    /// Number of bytes to buffer for writing.
    ///
    /// Defaults to the physical block size of the device (or 8192 if that is unset).
    #[clap(long)]
    buffer_size: Option<usize>,

    /// Random seed to use for generating random data. By default, this tool generates its own.
    #[clap(long)]
    seed: Option<u64>,

    /// Test the device even if the media type is not a spinning disk.
    #[clap(long)]
    allow_any_media: bool,

    /// Run the test even if the given path is a block device but not
    /// a disk (e.g. a single partition).
    #[clap(long)]
    allow_any_block_device: bool,
}

fn main() -> anyhow::Result<()> {
    let indicatif_layer = IndicatifLayer::new();
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(indicatif_layer.get_stderr_writer()))
        .with(indicatif_layer)
        .init();
    let args = Args::parse();
    let seed = args.seed.unwrap_or_else(|| thread_rng().gen());
    args.devices.into_par_iter().try_for_each(|device| {
        let ValidDevice {
            device,
            partition,
            path,
        } = device;
        let buffer_size = args.buffer_size.unwrap_or_else(|| {
            device
                .physical_block_size
                .unwrap_or(8192)
                .try_into()
                .unwrap()
        });

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
                anyhow::bail!("Device is not a rotational disk - this tool may be harmful to solid-state drives and others! Pass --allow-any-media to run anyway.");
            } else {
                warn!(?device.media_type, "Media type is not as expected but running tests anyway.");
            }
        }
        // TODO: Maybe test that the disk is empty?

        info!(?seed, ?partition, ?device, ?path, "Starting test");

        write_test::write(&path, buffer_size, seed).context("During write test")?;
        read_test::read_back(&path, buffer_size, seed).context("During read test")?;
        Ok(())
    })
}

lazy_static! {
    pub(crate) static ref PROGRESS_STYLE: ProgressStyle = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.white/grey} {bytes}/{total_bytes} ({bytes_per_sec}, ETA {eta_precise}) {msg}",
    ).expect("Internal error in indicatif progress bar template syntax");
}
