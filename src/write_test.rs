//! Running the "write" portion of the test.

use crate::{crypto::GarbageGenerator, PROGRESS_STYLE};
use anyhow::Context;
use std::{
    fs::OpenOptions,
    io::{self, BufReader, Seek},
    path::Path,
};
use tracing::{info_span, Span};
use tracing_indicatif::span_ext::IndicatifSpanExt;

#[tracing::instrument(skip(buffer_size, seed))]
pub(crate) fn write(
    dev_path: &Path,
    buffer_size: usize,
    seed: u64,
) -> anyhow::Result<()> {
    let mut out = OpenOptions::new()
        .write(true)
        .open(dev_path)
        .with_context(|| format!("Opening the device {:?} for writing", dev_path))?;
    let capacity = out.seek(io::SeekFrom::End(0))?;
    out.seek(io::SeekFrom::Start(0))?;

    let bar_span = info_span!("writing");
    bar_span.pb_set_style(&PROGRESS_STYLE);
    bar_span.pb_set_length(capacity);
    let _bar_span_handle = bar_span.enter();

    let generator = GarbageGenerator::new(buffer_size, seed, |read| {
        Span::current().pb_inc(read);
    });
    let mut generator = BufReader::new(generator);
    match io::copy(&mut generator, &mut out) {
        Ok(_) => Ok(()),
        Err(e) if e.raw_os_error() == Some(28) => {
            // "disk full", meaning we're done:
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}
