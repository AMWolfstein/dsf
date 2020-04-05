//! # dsf
//!
//! Utilities for working with DSF files. A DSF (DSD Stream File) is
//! an audio file which contains DSD audio data along with various
//! forms of metadata.

use id3::Tag;
use std::convert::TryFrom;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::prelude::*;
//use std::io::BufWriter;
use std::io::Read;
use std::io::SeekFrom;
//use std::io::Write;
use std::path::Path;
use std::u64;

/// In memory representation of a DSF file.
///
/// The [DSF File Format
/// Specification](https://dsd-guide.com/sites/default/files/white-papers/DSFFileFormatSpec_E.pdf)
/// divides a DSF file into four chunks:
///
/// - DSD chunk
/// - fmt chunk
/// - data chunk
/// - metadata chunk
///
/// The audio data is contained in the data chunk, everything else is
/// metadata about how the audio data should be interpretted or
/// metatadata about the audio itself e.g. Arist, Album, etc. So our
/// `Dsf` is made up of a `SampleData` struct which represents the
/// audio data, and a `Metadata` struct which represents everything
/// else.
pub struct DsfFile {
    file: File,
    dsd_chunk: DsdChunk,
    fmt_chunk: FmtChunk,
    data_chunk: DataChunk,
    //sample_data: SampleData,
    id3_tag: Tag,
}
impl DsfFile {
    // Open a DSF file.
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

        // id3v2 tag test
        file.seek(SeekFrom::Start(dsd_chunk.metadata_offset))?;
        let id3_tag = Tag::read_from(&file)?;
        // TODO: Change the way that we construct this so that it is neater
        //let sample_data = SampleData::new(file, &metadata)?;

        Ok(DsfFile {
            file,
            dsd_chunk,
            fmt_chunk,
            data_chunk,
            //sample_data,
            id3_tag,
        })
    }

    // TODO: fn interleave_samples_iter() -> InterleavedSamplesIter
    // TODO: fn channel_samples_iter(channelnum: u32) -> ChannelSamplesIter

    // TODO: Not sure this is the best place for this function
    // pub fn format_elapsed_of_total_duration(&self) -> String {
    //     let sampling_frequency = self.fmt_chunk.sampling_frequency as u64;
    //     let total_samples = self.fmt_chunk.sample_count;
    //     let total_seconds = total_samples / sampling_frequency;

    //     let duration_hours = total_seconds / 3600;
    //     let duration_minutes = (total_seconds / 60) - duration_hours * 60;
    //     let duration_seconds = total_seconds - (duration_hours * 3600 + duration_minutes * 60);

    //     let elapsed_seconds = (self.sample_data.get_fraction() * total_seconds as f64) as u64;
    //     let elapsed_hours = elapsed_seconds / 3600;
    //     let elapsed_minutes = (elapsed_seconds / 60) - elapsed_hours * 60;
    //     let elapsed_seconds = elapsed_seconds - (elapsed_hours * 3600 + elapsed_minutes * 60);

    //     format!(
    //         "{:02}:{:02}:{:02}/{:02}:{:02}:{:02}",
    //         elapsed_hours,
    //         elapsed_minutes,
    //         elapsed_seconds,
    //         duration_hours,
    //         duration_minutes,
    //         duration_seconds
    //     )
    // }
}
impl fmt::Display for DsfFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "DSD chunk:\n{}\n\nFmt chunk:\n{}\n\nData chunk:\n{}\n\nID3Tag:{:?}",
            self.dsd_chunk,
            self.fmt_chunk,
            self.data_chunk,
            self.id3_tag //self.sample_data
        )
    }
}

const DSD_CHUNK_HEADER: [u8; 4] = ['D' as u8, 'S' as u8, 'D' as u8, ' ' as u8];

struct DsdChunk {
    file_size: u64,
    metadata_offset: u64,
}
impl DsdChunk {
    fn new(file_size: u64, metadata_offset: u64) -> DsdChunk {
        DsdChunk {
            file_size,
            metadata_offset,
        }
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
        if &buffer[0..4] != DSD_CHUNK_HEADER {
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

const FMT_CHUNK_HEADER: [u8; 4] = ['f' as u8, 'm' as u8, 't' as u8, ' ' as u8];

struct FmtChunk {
    pub channel_type: ChannelType,
    pub channel_num: u32,
    pub sampling_frequency: u32,
    pub bits_per_sample: u32,
    pub sample_count: u64,
    pub block_size_per_channel: u32,
}
impl FmtChunk {
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

    fn format_duration(&self) -> String {
        let hours = self.sample_count / (self.sampling_frequency as u64 * 3600);
        let minutes = (self.sample_count / (self.sampling_frequency as u64 * 60)) - (hours * 60);
        let seconds =
            (self.sample_count / (self.sampling_frequency as u64)) - (hours * 3600 + minutes * 60);
        let samples = self.sample_count % (self.sampling_frequency as u64);

        format!("{:02}:{:02}:{:02};{}", hours, minutes, seconds, samples)
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
            self.format_duration()
        )
    }
}
impl TryFrom<[u8; 52]> for FmtChunk {
    type Error = Error;

    fn try_from(buffer: [u8; 52]) -> Result<Self, Self::Error> {
        if &buffer[0..4] != FMT_CHUNK_HEADER {
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
        if block_size_per_channel != BLOCK_SIZE as u32 {
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

enum ChannelType {
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
        let channel_type_as_str;

        match self {
            ChannelType::Mono => channel_type_as_str = "Mono",
            ChannelType::Stereo => channel_type_as_str = "Stereo: FL, FR.",
            ChannelType::ThreeChannels => channel_type_as_str = "3 channels: FL, FR, C.",
            ChannelType::Quad => channel_type_as_str = "Quad: FL, FR, BL, BR.",
            ChannelType::FourChannels => channel_type_as_str = "4 channels: FL, FR, C, LFE.",
            ChannelType::FiveChannels => channel_type_as_str = "5 channels: FL, FR, C, BL, BR.",
            ChannelType::FivePointOneChannels => {
                channel_type_as_str = "5.1 channels: FL, FR, C, LFE, BL, BR."
            }
        }

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

const DATA_CHUNK_HEADER: [u8; 4] = ['d' as u8, 'a' as u8, 't' as u8, 'a' as u8];

struct DataChunk {
    chunk_size: u64,
}
impl DataChunk {
    fn new(chunk_size: u64) -> DataChunk {
        DataChunk { chunk_size }
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
        if &buffer[0..4] != DATA_CHUNK_HEADER {
            return Err(Error::DataChunkHeader);
        }

        let chunk_size = u64_from_byte_buffer(&buffer, 4);

        Ok(DataChunk::new(chunk_size))
    }
}

// TODO go through below this line

/*
Nomencleture

DSD Sample Octete is a single byte in the DSD data. It consists of 8 DSD 1-bit samples.

dsopc_total = DSD Sample Octets Per Channel total: the total number of bytes in the DSD sample data per channel.
dsopc_index = DSD Sample Octets Per Channel index: the current index of the byte in the DSD sample data per channel (not per block).
channels_index = The current channel index = 0 to (channels_total - 1). Identifies the current channel we are working with.
channels_total = The number of channels e.g. 2 for stereo, 6 for 5.1 surround sound.
bf_index = Block-Frame index. From 0 to (bf_total - 1). The current block frame in the current_bf Vector.
bf_total = The total number of block-frames ceiling(total blocks / channels_total).
current_bf = A vector which contains the currently loaded block-frame. A vector of blocks, one for each channel of the current DSF frame.
bdso_index = Index of a DSD Sample Octet inside the current block.

*/

const BLOCK_SIZE: usize = 4096;
const DSD_SILENCE_BYTE: u8 = 0x69;
const DSF_SAMPLE_DATA_OFFSET: u64 = 92; //TODO check for off-by-one

pub struct SampleData {
    dsopc_total: u64,
    channel_total: usize,
    channel_index: usize,
    bf_total: u64,
    bf_index: u64,
    bf_size: usize,
    current_bf: Vec<[u8; BLOCK_SIZE]>,
    bdso_index: usize,
    bdso_max_for_last_bf: usize,
    last_bf: bool,
    end_of_data: bool,
    reverse_bits: bool,
    file: File,
}
impl SampleData {
    fn new(file: File, fmt_chunk: &FmtChunk, data_chunk: &DataChunk) -> Result<SampleData, Error> {
        let dsopc_total: u64 = fmt_chunk.sample_count / 8;

        let channel_total: usize = fmt_chunk.channel_num as usize;
        let channel_index: usize = 0;

        let dsd_sample_data_size: u64 = data_chunk.chunk_size - 12;
        let bf_total: u64 = (dsd_sample_data_size / BLOCK_SIZE as u64) / channel_total as u64;
        let bf_index: u64 = 0;
        let bf_size = BLOCK_SIZE * channel_total;

        let mut current_bf: Vec<[u8; BLOCK_SIZE]> = Vec::with_capacity(channel_total as usize);
        for _ in 0..channel_total {
            let block: [u8; BLOCK_SIZE] = [DSD_SILENCE_BYTE; BLOCK_SIZE];
            current_bf.push(block);
        }

        let bdso_index: usize = 0;
        let bdso_max_for_last_bf: usize = (dsopc_total % BLOCK_SIZE as u64) as usize;
        let last_bf: bool = false;
        let end_of_data = false;

        let reverse_bits: bool = fmt_chunk.bits_per_sample == 1;

        let mut dsf_sample_data: SampleData = SampleData {
            dsopc_total,
            channel_total,
            channel_index,
            bf_total,
            bf_index,
            bf_size,
            current_bf,
            bdso_index,
            bdso_max_for_last_bf,
            last_bf,
            end_of_data,
            reverse_bits,
            file,
        };

        dsf_sample_data.read_bf(0)?;

        Ok(dsf_sample_data)
    }

    pub fn get_fraction(&self) -> f64 {
        (self.bf_index * BLOCK_SIZE as u64 + self.bdso_index as u64) as f64
            / self.dsopc_total as f64
    }

    pub fn get_dsopc_index(&self) -> u64 {
        self.bf_index * BLOCK_SIZE as u64 + self.bdso_index as u64
    }

    pub fn get_dsopc_total(&self) -> u64 {
        self.dsopc_total
    }

    fn read_bf(&mut self, bf_index: u64) -> Result<bool, Error> {
        if self.bf_total == 0 || bf_index >= self.bf_total {
            return Ok(false);
        }

        let bf_offset = bf_index * self.bf_size as u64;

        self.file
            .seek(SeekFrom::Start(DSF_SAMPLE_DATA_OFFSET + bf_offset))?;

        for channel_index in 0..self.channel_total {
            let mut block: [u8; BLOCK_SIZE] = [0; BLOCK_SIZE];
            self.file.read_exact(&mut block)?;
            self.current_bf[channel_index] = block;
        }

        self.bf_index = bf_index;
        self.last_bf = self.bf_index + 1 == self.bf_total;
        self.bdso_index = 0;
        self.end_of_data = false;

        Ok(true)
    }

    fn read_next_bf(&mut self) -> Result<bool, Error> {
        if self.bf_index + 1 < self.bf_total {
            for channel_index in 0..self.channel_total {
                let mut block: [u8; BLOCK_SIZE] = [0; BLOCK_SIZE];
                self.file.read_exact(&mut block)?;
                self.current_bf[channel_index] = block;
            }

            self.bf_index += 1;
            self.last_bf = self.bf_index + 1 == self.bf_total;
            self.bdso_index = 0;

            return Ok(true);
        } else {
            return Ok(false);
        }
    }

    fn get_dso(&mut self, channel_index: usize, bdso_index: usize) -> u8 {
        if self.last_bf && bdso_index >= self.bdso_max_for_last_bf {
            if self.reverse_bits {
                return DSD_SILENCE_BYTE.reverse_bits();
            } else {
                return DSD_SILENCE_BYTE;
            }
        }

        if self.reverse_bits {
            return self.current_bf[channel_index][bdso_index].reverse_bits();
        } else {
            return self.current_bf[channel_index][bdso_index];
        }
    }

    fn increment_channel_index(&mut self) -> bool {
        if self.channel_index >= self.channel_total - 1 {
            self.channel_index = 0;
            return true;
        } else {
            self.channel_index += 1;
            return false;
        }
    }

    fn increment_bdso_index(&mut self, increment: usize) {
        if self.bdso_index + increment < BLOCK_SIZE {
            self.bdso_index += increment;
        } else {
            match self.read_next_bf() {
                Ok(success) => {
                    if !success {
                        self.end_of_data = true;
                    }
                }
                Err(_) => {
                    println!("ERROR: Failed to read next block-frame.");
                    self.end_of_data = true;
                }
            }
        }

        if self.last_bf && self.bdso_index > self.bdso_max_for_last_bf {
            self.end_of_data = true;
        }
    }
}
impl Iterator for SampleData {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.end_of_data {
            return None;
        }

        let u32be_byte_array: [u8; 4];

        u32be_byte_array = [
            self.get_dso(self.channel_index, self.bdso_index),
            self.get_dso(self.channel_index, self.bdso_index + 1),
            self.get_dso(self.channel_index, self.bdso_index + 2),
            self.get_dso(self.channel_index, self.bdso_index + 3),
        ];

        if self.increment_channel_index() {
            self.increment_bdso_index(4);
        }

        Some(u32::from_le_bytes(u32be_byte_array))
    }
}
impl fmt::Display for SampleData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Total DSD Sample Octets Per Channel = {}
Number of channels = {}
Number of block-frames = {}
Block-frame size = {} bytes
DSD samples in last block-frame = {}
Reverse bits = {}",
            self.dsopc_total,
            self.channel_total,
            self.bf_total,
            self.bf_size,
            self.bdso_max_for_last_bf,
            self.reverse_bits,
        )
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

fn u64_from_byte_buffer(buffer: &[u8], index: usize) -> u64 {
    let mut byte_array: [u8; 8] = [0; 8];
    byte_array.copy_from_slice(&buffer[index..index + 8]);

    u64::from_le_bytes(byte_array)
}

fn u32_from_byte_buffer(buffer: &[u8], index: usize) -> u32 {
    let mut byte_array: [u8; 4] = [0; 4];
    byte_array.copy_from_slice(&buffer[index..index + 4]);

    u32::from_le_bytes(byte_array)
}

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
