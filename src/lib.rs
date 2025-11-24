/*
 20251014: Changed by clone206 to prevent id3 errors from stopping processing of audio.

*/

//! DSF file utilities.
//!
//! A DSF (DSD Stream File) is a high-resolution audio file which
//! contains uncompressed DSD audio data along with information about
//! how the audio data is encoded. It can also optionally include an
//! [`ID3v2`](http://id3.org/) tag which contains metadata about the
//! music e.g. artist, album, etc.
//!
//! # Examples
//!
//! This example displays the metadata for the DSF file
//! `my/music.dsf`.
//!
//!```no_run
//! use dsf_meta::DsfFile;
//! use std::path::Path;
//!
//! let path = Path::new("my/music.dsf");
//!
//! match DsfFile::open(path) {
//!     Ok(dsf_file) => {
//!         println!("DSF file metadata:\n\n{}", dsf_file);
//!     }
//!     Err(error) => {
//!         println!("Error: {}", error);
//!     }
//! }
//! ```

// Get pedantic warnings when linting with `cargo clippy`.
#![warn(clippy::pedantic)]

mod id3_display;

use crate::id3_display::id3_tag_to_string;
use id3::Tag;
use sampled_data_duration::ConstantRateDuration;
use std::convert::TryFrom;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::Read;
use std::io::SeekFrom;
use std::path::Path;
use std::u64;

/// The offset in bytes of the sample data within a DSF file.
pub const DSF_SAMPLE_DATA_OFFSET: u64 = 92;

/// In memory representation of a DSF file.
///
/// The [DSF File Format
/// Specification](https://dsd-guide.com/sites/default/files/white-papers/DSFFileFormatSpec_E.pdf)
/// divides a DSF file into four chunks:
///
/// - [DSD chunk](struct.DsdChunk.html): basic file headers, size and pointers to other chunks.
/// - [Fmt chunk](struct.FmtChunk.html): information about the audio format e.g. sampling rate, etc.
/// - [Data chunk](struct.DataChunk.html): the audio samples.
/// - [Metadata chunk](id3::Tag): an optional `ID3v2` metadata tag.
///
/// The fields of the `DsfFile` struct reflect this specification,
/// with an additional [File] field for the underlying file,
/// and a [Tag read error](Option<Error>) field to store any error that occurred while reading the ID3 tag.
pub struct DsfFile {
    file: File,
    dsd_chunk: DsdChunk,
    fmt_chunk: FmtChunk,
    data_chunk: DataChunk,
    id3_tag: Option<Tag>,
    tag_read_err: Option<Error>,
}
impl DsfFile {
    /// Attempt to open and parse the metadata of DSF file in
    /// read-only mode. Sample data is not read into memory to keep
    /// the memory footprint small.
    ///
    /// # Errors
    ///
    /// This function will return an error if `path` does not exist or
    /// is not a readable and valid DSF file.
    ///
    /// # Examples
    ///
    ///```no_run
    /// use dsf_meta::DsfFile;
    /// use std::path::Path;
    ///
    /// let path = Path::new("my/music.dsf");
    ///
    /// match DsfFile::open(path) {
    ///     Ok(dsf_file) => {
    ///         println!("DSF file metadata:\n\n{}", dsf_file);
    ///     }
    ///     Err(error) => {
    ///         println!("Error: {}", error);
    ///     }
    /// }
    /// ```
    pub fn open(path: &Path) -> Result<DsfFile, Error> {
        let mut file = File::open(path)?;

        let mut dsd_chunk_buffer: [u8; 28] = [0; 28];
        file.read_exact(&mut dsd_chunk_buffer)?;
        let dsd_chunk = DsdChunk::try_from(dsd_chunk_buffer)?;

        let mut fmt_chunk_buffer: [u8; 52] = [0; 52];
        file.read_exact(&mut fmt_chunk_buffer)?;
        let fmt_chunk = FmtChunk::try_from(fmt_chunk_buffer)?;

        let mut data_chunk_buffer: [u8; 12] = [0; 12];
        file.read_exact(&mut data_chunk_buffer)?;
        let data_chunk = DataChunk::try_from(data_chunk_buffer)?;

        let mut dsf_file = DsfFile {
            file: file,
            dsd_chunk: dsd_chunk,
            fmt_chunk: fmt_chunk,
            data_chunk: data_chunk,
            id3_tag: None,
            tag_read_err: None
        };

        dsf_file.id3_tag = if dsf_file.dsd_chunk.metadata_offset == 0 {
            None
        } else {
            match dsf_file.file.seek(SeekFrom::Start(dsf_file.dsd_chunk.metadata_offset)) {
                Ok(_n) => match Tag::read_from2(&dsf_file.file) {
                    Ok(tag) => Some(tag),
                    Err(e) => {
                        let partial = e.partial_tag.clone();
                        dsf_file.tag_read_err = Some(Error::from(e));
                        partial
                    }
                },
                Err(e) => {
                    dsf_file.tag_read_err = Some(Error::from(e));
                    None
                }
            }
        };

        Ok(dsf_file)
    }

    /// Return a reference to the underlying [File]
    #[must_use]
    pub fn file(&self) -> &File {
        &self.file
    }

    /// Return a reference to the [`DsdChunk`](struct.DsdChunk.html).
    #[must_use]
    pub fn dsd_chunk(&self) -> &DsdChunk {
        &self.dsd_chunk
    }

    /// Return a reference to the [`FmtChunk`](struct.FmtChunk.html).
    #[must_use]
    pub fn fmt_chunk(&self) -> &FmtChunk {
        &self.fmt_chunk
    }

    /// Return a reference to the [`DataChunk`](struct.DataChunk.html).
    #[must_use]
    pub fn data_chunk(&self) -> &DataChunk {
        &self.data_chunk
    }

    /// Return a reference to the optional `ID3v2` [Tag]
    #[must_use]
    pub fn id3_tag(&self) -> &Option<Tag> {
        &self.id3_tag
    }

    /// Return the error if attempt to read the ID3 tag failed.
    pub fn tag_read_err(&self) -> Option<&Error> {
        self.tag_read_err.as_ref() 
    }
}
impl fmt::Display for DsfFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let id3_tag_as_string = match &self.id3_tag {
            Some(tag) => id3_tag_to_string(tag),
            None => String::from("None"),
        };
        write!(
            f,
            "DSD chunk:\n{}\n\nFmt chunk:\n{}\n\nData chunk:\n{}\n\nID3Tag:\n{}",
            self.dsd_chunk, self.fmt_chunk, self.data_chunk, &id3_tag_as_string,
        )
    }
}

/// Return a `u64` which starts from `index` in the specified byte
/// buffer, interpretting the bytes as little-endian.
fn u64_from_byte_buffer(buffer: &[u8], index: usize) -> u64 {
    let mut byte_array: [u8; 8] = [0; 8];
    byte_array.copy_from_slice(&buffer[index..index + 8]);

    u64::from_le_bytes(byte_array)
}

/// Return a `u32` which starts from `index` in the specified byte
/// buffer, interpretting the bytes as little-endian.
fn u32_from_byte_buffer(buffer: &[u8], index: usize) -> u32 {
    let mut byte_array: [u8; 4] = [0; 4];
    byte_array.copy_from_slice(&buffer[index..index + 4]);

    u32::from_le_bytes(byte_array)
}

/// The first chunk of a DSF file is the
/// [`DsdChunk`](struct.DsdChunk.html), which must begin with the
/// following four bytes.
const DSD_CHUNK_HEADER: [u8; 4] = [b'D', b'S', b'D', b' '];

/// The DSD chunk is the first chunk of a DSF file.
///
/// It contains the DSF file size and the offset of the `ID3v2` tag if
/// one exists.
pub struct DsdChunk {
    file_size: u64,
    metadata_offset: u64,
}
impl DsdChunk {
    /// Make a new `DsdChunk`.
    fn new(file_size: u64, metadata_offset: u64) -> DsdChunk {
        DsdChunk {
            file_size,
            metadata_offset,
        }
    }

    /// Returns the file size in bytes.
    #[must_use]
    pub fn file_size(&self) -> u64 {
        self.file_size
    }
}
impl fmt::Display for DsdChunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "File size = {} bytes\nMetadata offset = {} bytes",
            self.file_size, self.metadata_offset
        )
    }
}
impl TryFrom<[u8; 28]> for DsdChunk {
    type Error = Error;

    fn try_from(buffer: [u8; 28]) -> Result<Self, Self::Error> {
        if buffer[0..4] != DSD_CHUNK_HEADER {
            return Err(Error::DsdChunkHeader);
        }

        let chunk_size = u64_from_byte_buffer(&buffer, 4);
        if chunk_size != 28 {
            return Err(Error::DsdChunkSize);
        }

        let file_size = u64_from_byte_buffer(&buffer, 12);
        let metadata_offset = u64_from_byte_buffer(&buffer, 20);

        Ok(DsdChunk::new(file_size, metadata_offset))
    }
}

/// The first four bytes of the [`FmtChunk`](struct.FmtChunk.html).
const FMT_CHUNK_HEADER: [u8; 4] = [b'f', b'm', b't', b' '];

/// The fmt chunk contains information about the audio format.
///
/// - Channel type: mono, stereo, 5.1, etc.
/// - Channel number: 1 for mono, 2 for stereo, etc.
/// - Sampling frequency: the DSD sampling frequency.
/// - Bits per sample: whether the samples are big or little endian encoded.
/// - Block size per channel: this should always be 4096 bytes.
pub struct FmtChunk {
    channel_type: ChannelType,
    channel_num: u32,
    sampling_frequency: u32,
    bits_per_sample: u32,
    sample_count: u64,
    block_size_per_channel: u32,
}
impl FmtChunk {
    /// Make a new `FmtChunk`.
    fn new(
        channel_type: ChannelType,
        channel_num: u32,
        sampling_frequency: u32,
        bits_per_sample: u32,
        sample_count: u64,
        block_size_per_channel: u32,
    ) -> FmtChunk {
        FmtChunk {
            channel_type,
            channel_num,
            sampling_frequency,
            bits_per_sample,
            sample_count,
            block_size_per_channel,
        }
    }

    /// Return a reference to the
    /// [`ChannelType`](enum.ChannelType.html).
    #[must_use]
    pub fn channel_type(&self) -> &ChannelType {
        &self.channel_type
    }

    /// Return the number of channels. This should be in the range 1
    /// to 6.
    #[must_use]
    pub fn channel_num(&self) -> u32 {
        self.channel_num
    }

    /// Return the sampling freqency. DSD sampling frequencies are
    /// much higher than PCM because of the 1-bit sampling, so you
    /// should get values like:
    ///
    /// -  2822400 Hz for DSD64
    /// -  5644800 Hz for DSD128
    /// - 11289600 Hz for DSD256
    ///
    /// and so on.
    #[must_use]
    pub fn sampling_frequency(&self) -> u32 {
        self.sampling_frequency
    }

    /// Returns the `bits_per_sample` field. This is a bit of a
    /// misnomer in my opinion, but that’s what’s in the DSF
    /// specification. If it is equal to 1 then the sample data is
    /// stored least significant bit first. If it is equal to 8 then
    /// the sample data is stored most significant bit first.
    // TODO: Consider creating an endian-ness field instead to replace
    // this field
    #[must_use]
    pub fn bits_per_sample(&self) -> u32 {
        self.bits_per_sample
    }

    /// Return the number of DSD samples per channel.
    #[must_use]
    pub fn sample_count(&self) -> u64 {
        self.sample_count
    }

    /// Return the block size per channel in bytes. This is fixed and
    /// should always be 4096 bytes.
    #[must_use]
    pub fn block_size_per_channel(&self) -> u32 {
        self.block_size_per_channel
    }

    /// Return the duration of the audio.
    fn duration(&self) -> ConstantRateDuration {
        ConstantRateDuration::new(self.sample_count, u64::from(self.sampling_frequency))
    }
}
impl fmt::Display for FmtChunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Channel type = {}
Channel number = {}
Sampling frequency = {} Hz
Bits per sample = {}
Sample count per channel = {}
Block size per channel = {} bytes
Calculated duration = {} h:min:s;samples",
            self.channel_type,
            self.channel_num,
            self.sampling_frequency,
            self.bits_per_sample,
            self.sample_count,
            self.block_size_per_channel,
            self.duration()
        )
    }
}
impl TryFrom<[u8; 52]> for FmtChunk {
    type Error = Error;

    fn try_from(buffer: [u8; 52]) -> Result<Self, Self::Error> {
        if buffer[0..4] != FMT_CHUNK_HEADER {
            return Err(Error::FmtChunkHeader);
        }

        let chunk_size = u64_from_byte_buffer(&buffer, 4);
        if chunk_size != 52 {
            return Err(Error::FmtChunkSize);
        }

        let format_version = u32_from_byte_buffer(&buffer, 12);
        if format_version != 1 {
            return Err(Error::FormatVersion);
        }

        let format_id = u32_from_byte_buffer(&buffer, 16);
        if format_id != 0 {
            return Err(Error::FormatId);
        }

        let channel_type = ChannelType::try_from(u32_from_byte_buffer(&buffer, 20))?;

        let channel_num = u32_from_byte_buffer(&buffer, 24);
        match channel_num {
            1 | 2 | 3 | 4 | 5 | 6 => (),
            _ => return Err(Error::ChannelNum),
        }

        let sampling_frequency = u32_from_byte_buffer(&buffer, 28);
        let bits_per_sample = u32_from_byte_buffer(&buffer, 32);
        let sample_count = u64_from_byte_buffer(&buffer, 36);

        let block_size_per_channel = u32_from_byte_buffer(&buffer, 44);
        if block_size_per_channel != BLOCK_SIZE_AS_U32 {
            return Err(Error::BlockSizePerChannelNonStandard);
        }

        let reserved = u32_from_byte_buffer(&buffer, 48);
        if reserved != 0 {
            return Err(Error::ReservedNotZero);
        }

        Ok(FmtChunk::new(
            channel_type,
            channel_num,
            sampling_frequency,
            bits_per_sample,
            sample_count,
            block_size_per_channel,
        ))
    }
}

/// The different channel formats possible for a DSF file.
///
/// The channel specification is as follows:
///
/// <table style="empty-cells: hide;">
/// <tr><td></td><th style="text-align: center;" colspan="6">Channel Index</th></tr>
/// <tr><th>Channel Type</th><th>0</th><th>1</th><th>2</th><th>3</th><th>4</th><th>5</th></tr>
/// <tr><th>Mono</th>
///   <td>Center</td>
///   <td></td><td></td><td></td><td></td><td></td>
/// </tr>
/// <tr><th>Stereo</th>
///   <td>Front Left</td><td>Front Right</td>
///   <td></td><td></td><td></td><td></td>
/// </tr>
/// <tr><th>3-Channels</th>
///   <td>Front Left</td><td>Front Right</td><td>Center</td>
///   <td></td><td></td><td></td>
/// </tr>
/// <tr><th>Quad</th>
///   <td>Front Left</td><td>Front Right</td><td>Back Left</td>
///   <td>Back Right</td><td></td><td></td>
/// </tr>
/// <tr><th>4-Channels</th>
///   <td>Front Left</td><td>Front Right</td><td>Center</td>
///   <td>Low Frequency</td><td></td><td></td>
/// </tr>
/// <tr><th>5-Channels</th>
///   <td>Front Left</td><td>Front Right</td><td>Center</td>
///   <td>Back Left</td><td>Back Right</td><td></td>
/// </tr>
/// <tr><th>5.1-Channels</th>
///   <td>Front Left</td><td>Front Right</td><td>Center</td>
///   <td>Low Frequency</td><td>Back Left</td><td>Back Right</td>
/// </tr>
/// </table>
#[derive(Debug, Eq, PartialEq)]
pub enum ChannelType {
    Mono,
    Stereo,
    ThreeChannels,
    Quad,
    FourChannels,
    FiveChannels,
    FivePointOneChannels,
}
impl fmt::Display for ChannelType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let channel_type_as_str = match self {
            ChannelType::Mono => "Mono",
            ChannelType::Stereo => "Stereo: FL, FR.",
            ChannelType::ThreeChannels => "3 channels: FL, FR, C.",
            ChannelType::Quad => "Quad: FL, FR, BL, BR.",
            ChannelType::FourChannels => "4 channels: FL, FR, C, LFE.",
            ChannelType::FiveChannels => "5 channels: FL, FR, C, BL, BR.",
            ChannelType::FivePointOneChannels => "5.1 channels: FL, FR, C, LFE, BL, BR.",
        };

        write!(f, "{}", channel_type_as_str)
    }
}
impl TryFrom<u32> for ChannelType {
    type Error = Error;

    fn try_from(channel_type_as_u32: u32) -> Result<Self, Self::Error> {
        match channel_type_as_u32 {
            1 => Ok(ChannelType::Mono),
            2 => Ok(ChannelType::Stereo),
            3 => Ok(ChannelType::ThreeChannels),
            4 => Ok(ChannelType::Quad),
            5 => Ok(ChannelType::FourChannels),
            6 => Ok(ChannelType::FiveChannels),
            7 => Ok(ChannelType::FivePointOneChannels),
            _ => Err(Error::ChannelType),
        }
    }
}

/// First four bytes of the [`DataChunk`](struct.DataChunk.html).
const DATA_CHUNK_HEADER: [u8; 4] = [b'd', b'a', b't', b'a'];

/// The data chunk contains the DSD sample data.
pub struct DataChunk {
    chunk_size: u64,
}
impl DataChunk {
    /// Make a new `DataChunk`.
    fn new(chunk_size: u64) -> DataChunk {
        DataChunk { chunk_size }
    }

    /// The size of the data chunk in bytes. This is equal to the
    /// sample data + 12 bytes. The extra 12 bytes are taken up by the
    /// data chunk header and this size field.
    #[must_use]
    pub fn chunk_size(&self) -> u64 {
        self.chunk_size
    }
}
impl fmt::Display for DataChunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Data chunk size = {} bytes", self.chunk_size)
    }
}
impl TryFrom<[u8; 12]> for DataChunk {
    type Error = Error;

    fn try_from(buffer: [u8; 12]) -> Result<Self, Self::Error> {
        if buffer[0..4] != DATA_CHUNK_HEADER {
            return Err(Error::DataChunkHeader);
        }

        let chunk_size = u64_from_byte_buffer(&buffer, 4);

        Ok(DataChunk::new(chunk_size))
    }
}

/// The block size is always 4096 bytes.
const BLOCK_SIZE: usize = 4096;
/// We define another version of the block size as a `u32` because
/// when we try to simply cast a `usize` to a `u32` we get the
/// following pedantic clippy warning: *casting `usize` to `u32` may
/// truncate the value on targets with 64-bit wide
/// pointers*. Obviously, this is a false-positive because we are
/// dealing with a constant which is well within the range of a `u32`,
/// see clippy [issue
/// 7486](https://github.com/rust-lang/rust-clippy/issues/8316#issue-1109140641). However,
/// in the interests of getting a perfect report from clippy we add
/// the following constant:
const BLOCK_SIZE_AS_U32: u32 = 4096;

/// Errors provided by the dsf crate.
#[derive(Debug)]
pub enum Error {
    BlockSizePerChannelNonStandard,
    ChannelNum,
    ChannelType,
    DataChunkHeader,
    DsdChunkHeader,
    DsdChunkSize,
    FmtChunkHeader,
    FmtChunkSize,
    FormatId,
    FormatVersion,
    Id3Error(id3::Error),
    IoError(io::Error),
    ReservedNotZero,
    ChannelIndexOutOfRange,
    SampleIndexOutOfRange,
    FrameIndexOutOfRange,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::BlockSizePerChannelNonStandard => write!(
                f,
                "A fmt chunk is expected to specify its block size per channel as {}.",
                BLOCK_SIZE
            ),
            Error::ChannelNum => {
                f.write_str("A fmt chunk’s channel num is expected to be in the range 1–6.")
            }
            Error::ChannelType => {
                f.write_str("A fmt chunk’s channel type is expected to be in the range 1–7.")
            }
            Error::DataChunkHeader => f.write_str("A data chunk must start with the bytes 'data'."),
            Error::DsdChunkHeader => f.write_str("A DSD chunk must start with the bytes 'DSD '."),
            Error::DsdChunkSize => f.write_str("A DSD chunk must specify its size as 28 bytes."),
            Error::FmtChunkHeader => f.write_str("A fmt chunk must start with the bytes 'fmt '."),
            Error::FmtChunkSize => {
                f.write_str("A fmt chunk is expected to specify its size as 52 bytes.")
            }
            Error::FormatId => f.write_str("A fmt chumk must specifiy a format ID of 0."),
            Error::FormatVersion => f.write_str("A fmt chunk must specify version 1."),
            Error::Id3Error(id3_error) => write!(f, "Id3 error: {}", id3_error),
            Error::IoError(io_error) => write!(f, "IO error: {}", io_error),
            Error::ReservedNotZero => {
                f.write_str("A fmt chunk’s reserved space is expected to be zero filled.")
            }
            Error::ChannelIndexOutOfRange => f.write_str("Channel index is out of range."),
            Error::SampleIndexOutOfRange => f.write_str("Sample index is out of range."),
            Error::FrameIndexOutOfRange => f.write_str("Frame index is out of range."),
        }
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Id3Error(id3_error) => Some(id3_error),
            Error::IoError(io_error) => Some(io_error),
            _ => None,
        }
    }
}
impl From<id3::Error> for Error {
    fn from(error: id3::Error) -> Self {
        Error::Id3Error(error)
    }
}
impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IoError(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn get_sweep_dsf_file() -> Result<DsfFile, Error> {
        let sweep_filename = "sweep-176400hz-0-22050hz-20s-D64-2.8mhz.dsf";
        let path = Path::new(sweep_filename);

        if !path.is_file() {
            let sweep_url =
                "http://samplerateconverter.com/free-files/samples/dsf/sweep-176400hz-0-22050hz-20s-D64-2.8mhz.zip";

            Command::new("wget")
                .arg(sweep_url)
                .status()
                .unwrap_or_else(|_| panic!("Failed to download {}", sweep_url));

            let sweep_zip_filename = "sweep-176400hz-0-22050hz-20s-D64-2.8mhz.zip";
            Command::new("unzip")
                .arg(sweep_zip_filename)
                .status()
                .unwrap_or_else(|_| panic!("Failed to unzip {}", sweep_zip_filename));
        }

        DsfFile::open(path)
    }

    #[test]
    fn sweep_file() {
        let dsf_file = get_sweep_dsf_file().unwrap();

        assert_eq!(dsf_file.dsd_chunk.file_size, 14_114_908);
        assert_eq!(dsf_file.dsd_chunk.metadata_offset, 0);

        assert_eq!(dsf_file.fmt_chunk.channel_type, ChannelType::Stereo);
        assert_eq!(dsf_file.fmt_chunk.channel_num, 2);
        assert_eq!(dsf_file.fmt_chunk.sampling_frequency, 2_822_400);
        assert_eq!(dsf_file.fmt_chunk.bits_per_sample, 1);
        assert_eq!(dsf_file.fmt_chunk.sample_count, 56_459_264);
        assert_eq!(dsf_file.fmt_chunk.block_size_per_channel, 4096);
        // TODO: dsf_file.fmt_chunk.duration test

        assert_eq!(dsf_file.data_chunk.chunk_size, 14_114_828);

        // TODO: sample data test
    }
}
