use clap::Parser;
use std::path::PathBuf;
use tracing::{debug, error, info, span, warn, Level};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the device to test.
    ///
    /// This should be a mechanical disk block device (e.g. /dev/sda,
    /// /dev/disk/by-id/wwn-...).
    device: String,

    /// Test the device even if the device is not a spinning disk block device.
    #[clap(long)]
    ignore_device_mismatch: bool,
}

fn main() {
    let args = Args::parse();

    info!(device = args.device);
}
