//! Running the "write" portion of the test.

use crate::{buffer, PROGRESS_STYLE};
use anyhow::Context;
use std::{fs::OpenOptions, path::Path};
use tracing::{info_span, Span};
use tracing_indicatif::span_ext::IndicatifSpanExt;

#[tracing::instrument(skip(dev, buffer_size))]
pub(crate) fn run(
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
    let _bar_span_handle = bar_span.enter();

    let mut buf: Vec<u8> = Vec::with_capacity(buffer_size);
    buf.resize_with(buffer_size, Default::default);
    loop {
        let block = buffer::Block::new(buffer_size);
        match block.write_to(&mut out) {
            Ok(_) => {
                Span::current().pb_inc(buf.len().try_into().unwrap());
            }
            // When we encounter the "disk full" condition, quit.
            // TODO: Use ErrorKind::StorageFull when it's stable.
            Err(e) if e.raw_os_error() == Some(28) => {
                break;
            }
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}
