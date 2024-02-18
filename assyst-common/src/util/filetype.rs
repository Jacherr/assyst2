use std::cmp::min;
use std::ops::Range;

#[derive(Debug, PartialEq)]
pub enum Type {
    GIF,
    JPEG,
    PNG,
    WEBP,
    MP4,
    WEBM,
}
impl Type {
    pub fn as_str(&self) -> &'static str {
        match self {
            Type::GIF => "gif",
            Type::JPEG => "jpeg",
            Type::PNG => "png",
            Type::WEBP => "webp",
            Type::MP4 => "mp4",
            Type::WEBM => "webm",
        }
    }
    pub fn as_mime(&self) -> &'static str {
        match self {
            Type::GIF => "image/gif",
            Type::JPEG => "image/jpeg",
            Type::PNG => "image/png",
            Type::WEBP => "image/webp",
            Type::MP4 => "video/mp4",
            Type::WEBM => "video/webm",
        }
    }
    pub fn is_video(&self) -> bool {
        match self {
            Type::MP4 | Type::WEBM => true,
            _ => false,
        }
    }
}

const WEBP: [u8; 4] = [87, 69, 66, 80];
const MP4: [u8; 4] = [0x66, 0x74, 0x79, 0x70];

fn bounded_range(start: usize, end: usize, len: usize) -> Range<usize> {
    min(len, start)..min(len, end)
}

fn sig(that: &[u8], eq: &[u8]) -> bool {
    that[0..std::cmp::min(eq.len(), that.len())].eq(eq)
}

fn check_webp(that: &[u8]) -> bool {
    let bytes_offset_removed = &that[bounded_range(8, 12, that.len())];
    sig(bytes_offset_removed, &WEBP)
}

fn check_mp4(that: &[u8]) -> bool {
    let bytes_offset_removed = &that[bounded_range(4, 8, that.len())];
    sig(bytes_offset_removed, &MP4)
}

pub fn get_sig(buf: &[u8]) -> Option<Type> {
    match buf {
        [71, 73, 70, ..] => Some(Type::GIF),
        [255, 216, 255, ..] => Some(Type::JPEG),
        [137, 80, 78, 71, 13, 10, 26, 10, ..] => Some(Type::PNG),
        [0x1A, 0x45, 0xDF, 0xA3] => Some(Type::WEBM),
        _ if check_webp(buf) => Some(Type::WEBP),
        _ if check_mp4(buf) => Some(Type::MP4),
        _ => None,
    }
}
