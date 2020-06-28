# tabfile

A rust library for convenient reading of tab-delimited files.

Please see the [docs.rs documentation](https://docs.rs/tabfile/0.2.1/tabfile/) for usage information.

## Differences to the `csv` crate

When I created this crate I was not aware of the [csv crate](https://crates.io/crates/csv).
The csv crate offers the same functionality and is probably a better choice in most cases.
The only missing feature seems to be the unconditional skipping of lines from the beginning of a file (which is sometimes useful when you are dealing with ad-hoc file formats).
If you care about having fewer dependencies, then `tabfile` might be a better choice, because it is completely self-contained.
