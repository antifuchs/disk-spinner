# disk-spinner - a stress-test tool for spinning rust HDDs

If you set up a new mechanical disk that you take fresh out of the package and slot it into your NAS, do you ask yourself whether the data on it will be safe? I do, and so do a bunch of folks who run machines with lots of drives!

There's a pretty decent procedure that I'd been using to burn-in my HDDs, taken from [this forum thread](https://www.truenas.com/community/resources/hard-drive-burn-in-testing.92/); this is fine but these days, HDDs in excess of 16TiB exist, and on those badblocks runs into a limitation of its block offset representation.

Hence, this tool: disk-spinner.

## What does this do?

It destructively writes blocks of random data with a checksum to an entire disk device (or, optionally, just a partition; but you'll probably want the whole drive), then verifies that the data matches the checksum.

If any data could not be read exactly as written, it informs you in big letters. That means your disk is bad & you should make use of your vendor's RMA policy. Doesn't it feel great to not run into problems?

But note: **THE VERIFICATION BIT IS NOT IMPLEMENTED YET**.

## The name

This tool is for spinning disks; it's also a play on the German word "Spinner" (a goofball), referring to me - a person goofy about disks.
