#[macro_use]
extern crate lazy_static;

use indicatif::ProgressStyle;
use tracing::Span;
use tracing_indicatif::span_ext::IndicatifSpanExt;
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use anyhow::Context;
use clap::Parser;
use std::{
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
    str::FromStr,
};
use tracing::{info, info_span, warn};

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

    /// Number of bytes to buffer for writing.
    ///
    /// Defaults to the physical block size of the device (or 8192 if that is unset).
    #[clap(long)]
    buffer_size: Option<usize>,

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
            anyhow::bail!("Device is not a rotational disk - this tool may be harmful to solid-state drives and others! Pass --allow-any-media to run anyway.");
        } else {
            warn!(?device.media_type, "Media type is not as expected but running tests anyway.");
        }
    }
    // TODO: Maybe test that the disk is empty?

    info!(?partition, ?device, ?path, "Starting test");

    write_test(
        &path,
        &device,
        args.buffer_size.unwrap_or_else(|| {
            device
                .physical_block_size
                .unwrap_or(8192)
                .try_into()
                .unwrap()
        }),
    )
    .context("Write test")?;
    Ok(())
}

lazy_static! {
    static ref PROGRESS_STYLE: ProgressStyle = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.white/grey} {bytes}/{total_bytes} ({bytes_per_sec}, ETA {eta_precise}) {msg}",
    ).expect("Internal error in indicatif progress bar template syntax");
}

// TODO: return hashed data
#[tracing::instrument(skip(dev, buffer_size))]
fn write_test(
    dev_path: &Path,
    dev: &block_utils::Device,
    buffer_size: usize,
) -> anyhow::Result<()> {
    let mut out = OpenOptions::new()
        .write(true)
        .open(dev_path)
        .with_context(|| format!("Opening the device {:?} for writing", dev_path))?;

    let bar_span = info_span!("writing");
    bar_span.pb_set_style(&PROGRESS_STYLE);
    bar_span.pb_set_length(dev.capacity);
    let bar_span_handle = bar_span.enter();

    let mut buf: Vec<u8> = Vec::with_capacity(buffer_size);
    loop {
        for _n in 0..200 {
            buf.write_all(&[0xca, 0xfe, 0xba, 0xbe])
                .context("filling buffer")?;
        }
        out.write_all(&buf)?; // feca beba for some reason
        Span::current().pb_inc(buf.len().try_into().unwrap());
        buf.clear();
    }

    Ok(())
}
