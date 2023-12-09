//! Running the "write" portion of the test.

use crate::{buffer, PROGRESS_STYLE};
use anyhow::Context;
use std::{
    fs::OpenOptions,
    num::NonZeroUsize,
    path::Path,
    sync::mpsc::{channel, Sender},
    thread::{self, JoinHandle},
};
use tracing::{info_span, trace, Span};
use tracing_indicatif::span_ext::IndicatifSpanExt;

#[tracing::instrument(skip(dev, buffer_size, concurrency))]
pub(crate) fn run(
    dev_path: &Path,
    dev: &block_utils::Device,
    buffer_size: usize,
    concurrency: Option<NonZeroUsize>,
) -> anyhow::Result<()> {
    let mut out = OpenOptions::new()
        .write(true)
        .open(dev_path)
        .with_context(|| format!("Opening the device {:?} for writing", dev_path))?;

    let (tx, rx) = channel();
    let _handles = generate_all_blocks(&tx, buffer_size, concurrency);

    let bar_span = info_span!("writing");
    bar_span.pb_set_style(&PROGRESS_STYLE);
    bar_span.pb_set_length(dev.capacity);
    let _bar_span_handle = bar_span.enter();

    let mut buf: Vec<u8> = Vec::with_capacity(buffer_size);
    buf.resize_with(buffer_size, Default::default);
    loop {
        let block = rx.recv().context("Could not receive to-write block")?;
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

fn generate_all_blocks(
    sender: &Sender<buffer::Block>,
    block_size: usize,
    concurrency: Option<NonZeroUsize>,
) -> anyhow::Result<Vec<JoinHandle<()>>> {
    let concurrency = if let Some(c) = concurrency {
        c
    } else {
        thread::available_parallelism()?
    };
    let handles = (0..concurrency.get()).enumerate().map(|_| {
        let sender = sender.clone();
        thread::spawn(move || generate_block(sender, block_size))
    });
    Ok(handles.collect())
}

fn generate_block(sender: Sender<buffer::Block>, block_size: usize) {
    loop {
        if let Err(error) = sender.send(buffer::Block::new(block_size)) {
            trace!(%error, "Received error sending down the to-write block channel. Treating as our signal to terminate.");
            return;
        }
    }
}
