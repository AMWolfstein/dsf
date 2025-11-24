<!-- markdownlint-disable MD033 -->
# <img src="doc/images/logo.png" width="100" alt="Logo"/> DSD Stream File #

DSF (DSD Stream File) support in Rust. DSF files are a high-resolution
audio format that contain lossless 1-bit audio stream in delta sigma
modulation aka Direct Stream Digital (DSD). The format is intended for
1-bit DSD DACs.

This package is a mostly compatible fork of the dsf crate by Daniel J. R. May, but it focuses on reading metadata and leaves it up to the implementor to iterate over the dsd samples using the info. It also allows for incomplete ID3 tag reads without causing a failure to instantiate the DsfFile object.

## References ##

* [DSF file format
specification](https://dsd-guide.com/sites/default/files/white-papers/DSFFileFormatSpec_E.pdf)
