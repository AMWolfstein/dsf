# dsf TODOs #

* [x] Rename `dsf-probe` to `dsf-metatadata`.
* [x] Refactor dsf samples iterater.
* [x] Create getters for structs, rather than public members.
* [x] Add documentation.
* [x] Create a audio samples duration struct and trait implementations?
* [ ] Migrate `dsf-metadata` to `dsd` crate, and improve it.
* [ ] Implement a iterator for a single specific channel to allow
      channel mixing and monitoring.
* [ ] Migrate `id3::Tag` display code to `id3` crate? Have opened an
      [issue](https://github.com/polyfloyd/rust-id3/issues/37) about this.
* [ ] Consider an `endianness` method on `FmtChunk` which would return
      a `bool` as this is more expressive than `bits_per_sample`.
* [ ] Improve tests, automatically download dsf files to run on from
      an archive so we don’t have large files in the git repository.
* [ ] Add a silence iterator so that we can generate silence of
      specified length.
* [ ] Add GitLab CI script.
