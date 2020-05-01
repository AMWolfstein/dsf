//! ID3v2 Tag display module.
//!
//! This module contains functions which can be used to get a _human
//! readable_ `String` representation of a `id3::Tag`.

use id3::Tag;

/// Return a `String` representation of the `id3::Tag`.
pub fn id3_tag_to_string(tag: &Tag) -> String {
    let mut s = String::new();
    for frame in tag.frames() {
        let id: &str = frame.id();
        let name: &str = frame.name();
        let content: &str = &id3_content_to_string(frame.content());
        s.push_str(&format!("{}: {} = {}\n", id, name, content));
    }

    s
}

use id3::Content;

fn id3_content_to_string(content: &Content) -> String {
    match content {
        Content::Text(text) => format!("Text: {}", text),
        Content::Link(link) => format!("Link: {}", link),
        Content::ExtendedText(ext_text) => format!(
            "Extended text:\n\tDescription = {}\n\tValue = {}",
            ext_text.description, ext_text.value
        ),
        Content::ExtendedLink(ext_link) => format!(
            "Extended link:\n\tDescription = {}\n\tLink = {}",
            ext_link.description, ext_link.link
        ),
        Content::Comment(comment) => format!(
            "Comment:\n\tLanguage = {}\n\tDescription = {}\n\tText = {}",
            comment.lang, comment.description, comment.text
        ),
        Content::Lyrics(lyrics) => format!(
            "Lyrics:\n\tLanguage = {}\n\tDescription = {}\n\tText = {}",
            lyrics.lang, lyrics.description, lyrics.text
        ),
        Content::SynchronisedLyrics(syn_lyrics) => id3_synchronised_lyrics_to_string(syn_lyrics),
        Content::Picture(picture) => id3_picture_to_string(picture),
        Content::Unknown(data) => format!("Unknown: {} bytes", data.len()),
    }
}

const MILLISECONDS_PER_HOUR: u32 = 3600000;
const MILLISECONDS_PER_MINUTE: u32 = 60000;
const MILLISECONDS_PER_SECOND: u32 = 1000;

use id3::frame::{SynchronisedLyrics, TimestampFormat};
fn id3_synchronised_lyrics_to_string(syn_lyrics: &SynchronisedLyrics) -> String {
    let mut s = format!(
        "Synchronised lyrics:\n\tLanguage = {}\n\tTimestamp format = {}\n\tContent type = {}\n\tContent:\n",
        syn_lyrics.lang,
        syn_lyrics.timestamp_format,
        syn_lyrics.content_type,
    );

    match syn_lyrics.timestamp_format {
        TimestampFormat::MPEG => {
            s.push_str(&format!("\tFrame\t{}\n", syn_lyrics.content_type));

            for (frame, lyric) in syn_lyrics.content.iter() {
                s.push_str(&format!("\t{}\t{}\n", frame, lyric));
            }
        }
        TimestampFormat::MS => {
            s.push_str(&format!("\tTimecode\t{}\n", syn_lyrics.content_type));

            for (total_ms, lyric) in syn_lyrics.content.iter() {
                let hours = total_ms / MILLISECONDS_PER_HOUR;
                let mins = (total_ms % MILLISECONDS_PER_HOUR) / MILLISECONDS_PER_MINUTE;
                let secs = (total_ms % MILLISECONDS_PER_MINUTE) / MILLISECONDS_PER_SECOND;
                let ms = total_ms % MILLISECONDS_PER_SECOND;

                s.push_str(&format!(
                    "\t{:02}:{:02}:{:02}.{:03}\t{}\n",
                    hours, mins, secs, ms, lyric
                ))
            }
        }
    }
    s
}

use id3::frame::Picture;
fn id3_picture_to_string(picture: &Picture) -> String {
    format!(
        "Picture:\n\tType = {}\n\tDescription = {}\n\tMIME type = {}\n\tSize = {} bytes",
        picture.picture_type,
        picture.description,
        picture.mime_type,
        picture.data.len()
    )
}
