//! ID3v2 Tag display module.
//!
//! This module contains functions which can be used to get a _human
//! readable_ `String` representation of a `id3::Tag`.
//!
//! The functionality this module provides may be merged into the
//! `id3` crate at some point in the future so they are kept in this
//! separte file.

use id3::Tag;

/// Return a `String` representation of the `id2::Tag`.
pub fn id3_tag_to_string(tag: &Tag) -> String {
    let mut s = String::new();
    for frame in tag.frames() {
        let id: &str = frame.id();
        let name: &str = id3_frame_name(id);
        let content: &str = &id3_content_to_string(frame.content());
        s.push_str(&format!("{}: {} = {}\n", id, name, content));
    }

    s
}

use id3::Content;

fn id3_content_to_string(content: &Content) -> String {
    match content {
        Content::Text(text) => text.to_string(),
        Content::ExtendedText(extended_text) => id3_extended_text_to_string(extended_text),
        Content::Link(link) => link.to_string(),
        Content::ExtendedLink(extended_link) => id3_extended_link_to_string(extended_link),
        Content::Comment(comment) => id3_comment_to_string(comment),
        Content::Lyrics(lyrics) => id3_lyrics_to_string(lyrics),
        Content::SynchronisedLyrics(syn_lyrics) => id3_synchronised_lyrics_to_string(syn_lyrics),
        Content::Picture(picture) => id3_picture_to_string(picture),
        Content::Unknown(_vec_u8) => String::from("Unknown"),
    }
}

use id3::frame::ExtendedText;
fn id3_extended_text_to_string(etext: &ExtendedText) -> String {
    format!("{}: {}", etext.description, etext.value)
}

use id3::frame::ExtendedLink;
fn id3_extended_link_to_string(elink: &ExtendedLink) -> String {
    format!("{} {}", elink.description, elink.link)
}

use id3::frame::Comment;
fn id3_comment_to_string(comment: &Comment) -> String {
    format!("{}:{}:{}", comment.lang, comment.description, comment.text)
}

use id3::frame::Lyrics;
fn id3_lyrics_to_string(lyrics: &Lyrics) -> String {
    format!("{}:{}:{}", lyrics.lang, lyrics.description, lyrics.text)
}

use id3::frame::SynchronisedLyrics;
fn id3_synchronised_lyrics_to_string(syn_lyrics: &SynchronisedLyrics) -> String {
    let mut s = format!(
        "Lang: {}\nTimestamp format: {}\nType: {}\n",
        syn_lyrics.lang,
        id3_timestamp_format_to_str(&syn_lyrics.timestamp_format),
        id3_synchronised_lyrics_type_to_str(&syn_lyrics.content_type),
    );

    for (timecode, line) in syn_lyrics.content.iter() {
        s.push_str(&timecode.to_string()); // TODO: Convert timecode to string
        s.push_str(": ");
        s.push_str(&line);
        s.push_str("\n");
    }

    s
}

use id3::frame::TimestampFormat;
fn id3_timestamp_format_to_str(tstamp_fmt: &TimestampFormat) -> &'static str {
    match tstamp_fmt {
        TimestampFormat::MPEG => "MPEG",
        TimestampFormat::MS => "MS",
    }
}

use id3::frame::SynchronisedLyricsType;
fn id3_synchronised_lyrics_type_to_str(syn_lyrics_type: &SynchronisedLyricsType) -> &'static str {
    match syn_lyrics_type {
        SynchronisedLyricsType::Other => "Other",
        SynchronisedLyricsType::Lyrics => "Lyrics",
        SynchronisedLyricsType::Transcription => "Transcription",
        SynchronisedLyricsType::PartName => "Part Name",
        SynchronisedLyricsType::Event => "Event",
        SynchronisedLyricsType::Chord => "Chord",
        SynchronisedLyricsType::Trivia => "Trivia",
    }
}

use id3::frame::Picture;
fn id3_picture_to_string(picture: &Picture) -> String {
    format!(
        "\n\tFormat: {}\n\tType: {}\n\tDescription: {}\n\tSize: {} bytes",
        picture.mime_type,
        id3_picture_type_to_str(picture.picture_type),
        picture.description,
        picture.data.len()
    )
}

use id3::frame::PictureType;
fn id3_picture_type_to_str(picture_type: PictureType) -> &'static str {
    match picture_type {
        PictureType::Other => "Other",
        PictureType::Icon => "Icon",
        PictureType::OtherIcon => "Other Icon",
        PictureType::CoverFront => "Cover Front",
        PictureType::CoverBack => "Cover Back",
        PictureType::Leaflet => "Leaflet",
        PictureType::Media => "Media",
        PictureType::LeadArtist => "Lead Artist",
        PictureType::Artist => "Artist",
        PictureType::Conductor => "Conductor",
        PictureType::Band => "Band",
        PictureType::Composer => "Composer",
        PictureType::Lyricist => "Lyricist",
        PictureType::RecordingLocation => "Recording Location",
        PictureType::DuringRecording => "During Recording",
        PictureType::DuringPerformance => "During Performance",
        PictureType::ScreenCapture => "Screen Capture",
        PictureType::BrightFish => "Bright Fish",
        PictureType::Illustration => "Illustration",
        PictureType::BandLogo => "Band Logo",
        PictureType::PublisherLogo => "Publisher Logo",
        PictureType::Undefined(_) => "Undefined",
    }
}

fn id3_frame_name(id: &str) -> &'static str {
    match id {
        // Ids and names defined in section 4 of http://id3.org/id3v2.4.0-frames
        "AENC" => "Audio encryption",
        "APIC" => "Attached picture",
        "ASPI" => "Audio seek point index",
        "COMM" => "Comments",
        "COMR" => "Commercial frame",
        "ENCR" => "Encryption method registration",
        "EQU2" => "Equalisation (2)",
        "ETCO" => "Event timing codes",
        "GEOB" => "General encapsulated object",
        "GRID" => "Group identification registration",
        "LINK" => "Linked information",
        "MCDI" => "Music CD identifier",
        "MLLT" => "MPEG location lookup table",
        "OWNE" => "Ownership frame",
        "PRIV" => "Private frame",
        "PCNT" => "Play counter",
        "POPM" => "Popularimeter",
        "POSS" => "Position synchronisation frame",
        "RBUF" => "Recommended buffer size",
        "RVA2" => "Relative volume adjustment (2)",
        "RVRB" => "Reverb",
        "SEEK" => "Seek frame",
        "SIGN" => "Signature frame",
        "SYLT" => "Synchronised lyric/text",
        "SYTC" => "Synchronised tempo codes",
        "TALB" => "Album/Movie/Show title",
        "TBPM" => "BPM (beats per minute)",
        "TCOM" => "Composer",
        "TCON" => "Content type",
        "TCOP" => "Copyright message",
        "TDEN" => "Encoding time",
        "TDLY" => "Playlist delay",
        "TDOR" => "Original release time",
        "TDRC" => "Recording time",
        "TDRL" => "Release time",
        "TDTG" => "Tagging time",
        "TENC" => "Encoded by",
        "TEXT" => "Lyricist/Text writer",
        "TFLT" => "File type",
        "TIPL" => "Involved people list",
        "TIT1" => "Content group description",
        "TIT2" => "Title/songname/content description",
        "TIT3" => "Subtitle/Description refinement",
        "TKEY" => "Initial key",
        "TLAN" => "Language(s)",
        "TLEN" => "Length",
        "TMCL" => "Musician credits list",
        "TMED" => "Media type",
        "TMOO" => "Mood",
        "TOAL" => "Original album/movie/show title",
        "TOFN" => "Original filename",
        "TOLY" => "Original lyricist(s)/text writer(s)",
        "TOPE" => "Original artist(s)/performer(s)",
        "TOWN" => "File owner/licensee",
        "TPE1" => "Lead performer(s)/Soloist(s)",
        "TPE2" => "Band/orchestra/accompaniment",
        "TPE3" => "Conductor/performer refinement",
        "TPE4" => "Interpreted, remixed, or otherwise modified by",
        "TPOS" => "Part of a set",
        "TPRO" => "Produced notice",
        "TPUB" => "Publisher",
        "TRCK" => "Track number/Position in set",
        "TRSN" => "Internet radio station name",
        "TRSO" => "Internet radio station owner",
        "TSOA" => "Album sort order",
        "TSOP" => "Performer sort order",
        "TSOT" => "Title sort order",
        "TSRC" => "ISRC (international standard recording code)",
        "TSSE" => "Software/Hardware and settings used for encoding",
        "TSST" => "Set subtitle",
        "TXXX" => "User defined text information frame",
        "UFID" => "Unique file identifier",
        "USER" => "Terms of use",
        "USLT" => "Unsynchronised lyric/text transcription",
        "WCOM" => "Commercial information",
        "WCOP" => "Copyright/Legal information",
        "WOAF" => "Official audio file webpage",
        "WOAR" => "Official artist/performer webpage",
        "WOAS" => "Official audio source webpage",
        "WORS" => "Official Internet radio station homepage",
        "WPAY" => "Payment",
        "WPUB" => "Publishers official webpage",
        "WXXX" => "User defined URL link frame",

        // Ids and names defined in section 4 of
        // http://id3.org/d3v2.3.0 which have not been previously
        // defined above
        "EQUA" => "Equalization",
        "IPLS" => "Involved people list",
        "RVAD" => "Relative volume adjustment",
        "TDAT" => "Date",
        "TIME" => "Time",
        "TORY" => "Original release year",
        "TRDA" => "Recording dates",
        "TSIZ" => "Size",
        "TYER" => "Year",

        // Ids and names defined in section 4 of
        // http://id3.org/d3v2-00 which have not been previously
        // defined above
        "BUF" => "Recommended buffer size",
        "CNT" => "Play counter",
        "COM" => "Comments",
        "CRA" => "Audio encryption",
        "CRM" => "Encrypted meta frame",
        "ETC" => "Event timing codes",
        "EQU" => "Equalization",
        "GEO" => "General encapsulated object",
        "IPL" => "Involved people list",
        "LNK" => "Linked information",
        "MCI" => "Music CD Identifier",
        "MLL" => "MPEG location lookup table",
        "PIC" => "Attached picture",
        "POP" => "Popularimeter",
        "REV" => "Reverb",
        "RVA" => "Relative volume adjustment",
        "SLT" => "Synchronized lyric/text",
        "STC" => "Synced tempo codes",
        "TAL" => "Album/Movie/Show title",
        "TBP" => "BPM (Beats Per Minute)",
        "TCM" => "Composer",
        "TCO" => "Content type",
        "TCR" => "Copyright message",
        "TDA" => "Date",
        "TDY" => "Playlist delay",
        "TEN" => "Encoded by",
        "TFT" => "File type",
        "TIM" => "Time",
        "TKE" => "Initial key",
        "TLA" => "Language(s)",
        "TLE" => "Length",
        "TMT" => "Media type",
        "TOA" => "Original artist(s)/performer(s)",
        "TOF" => "Original filename",
        "TOL" => "Original Lyricist(s)/text writer(s)",
        "TOR" => "Original release year",
        "TOT" => "Original album/Movie/Show title",
        "TP1" => "Lead artist(s)/Lead performer(s)/Soloist(s)/Performing group",
        "TP2" => "Band/Orchestra/Accompaniment",
        "TP3" => "Conductor/Performer refinement",
        "TP4" => "Interpreted, remixed, or otherwise modified by",
        "TPA" => "Part of a set",
        "TPB" => "Publisher",
        "TRC" => "ISRC (International Standard Recording Code)",
        "TRD" => "Recording dates",
        "TRK" => "Track number/Position in set",
        "TSI" => "Size",
        "TSS" => "Software/hardware and settings used for encoding",
        "TT1" => "Content group description",
        "TT2" => "Title/Songname/Content description",
        "TT3" => "Subtitle/Description refinement",
        "TXT" => "Lyricist/text writer",
        "TXX" => "User defined text information frame",
        "TYE" => "Year",
        "UFI" => "Unique file identifier",
        "ULT" => "Unsychronized lyric/text transcription",
        "WAF" => "Official audio file webpage",
        "WAR" => "Official artist/performer webpage",
        "WAS" => "Official audio source webpage",
        "WCM" => "Commercial information",
        "WCP" => "Copyright/Legal information",
        "WPB" => "Publishers official webpage",
        "WXX" => "User defined URL link frame",

        _ => "Unknown",
    }
}
