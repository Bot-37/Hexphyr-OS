use core::{slice, str};

const HEADER_LEN: usize = 110;
const MAGIC_NEWC: &[u8; 6] = b"070701";
const TRAILER_NAME: &str = "TRAILER!!!";

pub struct Archive<'a> {
    bytes: &'a [u8],
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Summary {
    pub entry_count: usize,
    pub payload_bytes: usize,
    pub has_init: bool,
    pub has_shell: bool,
    pub has_issue: bool,
}

struct Header {
    namesize: usize,
    filesize: usize,
}

pub struct Entry<'a> {
    pub name: &'a str,
    pub data: &'a [u8],
}

impl<'a> Archive<'a> {
    pub unsafe fn from_raw(ptr: *const u8, len: usize) -> Option<Self> {
        if ptr.is_null() || len < HEADER_LEN {
            return None;
        }

        // SAFETY: The bootloader passes an in-memory initramfs blob with a
        // stable lifetime for the entire kernel session.
        let bytes = unsafe { slice::from_raw_parts(ptr, len) };
        Some(Self { bytes })
    }

    pub fn summary(&self) -> Summary {
        let mut summary = Summary::default();
        let mut offset = 0usize;

        while let Some((entry, next_offset)) = self.entry_at(offset) {
            if entry.name == TRAILER_NAME {
                break;
            }

            summary.entry_count += 1;
            summary.payload_bytes = summary.payload_bytes.saturating_add(entry.data.len());
            summary.has_init |= matches_path(entry.name, "sbin/init");
            summary.has_shell |= matches_path(entry.name, "bin/sh");
            summary.has_issue |= matches_path(entry.name, "etc/issue");

            offset = next_offset;
        }

        summary
    }

    fn entry_at(&self, offset: usize) -> Option<(Entry<'a>, usize)> {
        let header_bytes = self.bytes.get(offset..offset.checked_add(HEADER_LEN)?)?;
        if &header_bytes[..MAGIC_NEWC.len()] != MAGIC_NEWC {
            return None;
        }

        let header = Header {
            namesize: read_hex_field(header_bytes, 94, 8)?,
            filesize: read_hex_field(header_bytes, 54, 8)?,
        };

        if header.namesize == 0 {
            return None;
        }

        let name_start = offset.checked_add(HEADER_LEN)?;
        let name_end = name_start.checked_add(header.namesize)?;
        let raw_name = self.bytes.get(name_start..name_end)?;
        let name = parse_name(raw_name)?;

        let data_start = align_up(name_end, 4)?;
        let data_end = data_start.checked_add(header.filesize)?;
        let data = self.bytes.get(data_start..data_end)?;
        let next_offset = align_up(data_end, 4)?;

        Some((Entry { name, data }, next_offset))
    }
}

fn parse_name(raw_name: &[u8]) -> Option<&str> {
    let nul = raw_name.iter().position(|byte| *byte == 0)?;
    str::from_utf8(raw_name.get(..nul)?).ok()
}

fn read_hex_field(bytes: &[u8], start: usize, len: usize) -> Option<usize> {
    let field = bytes.get(start..start.checked_add(len)?)?;
    let mut value = 0usize;
    for byte in field {
        value = value.checked_mul(16)?;
        value = value.checked_add(hex_value(*byte)? as usize)?;
    }
    Some(value)
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn align_up(value: usize, align: usize) -> Option<usize> {
    let mask = align.checked_sub(1)?;
    value.checked_add(mask).map(|aligned| aligned & !mask)
}

fn matches_path(actual: &str, expected: &str) -> bool {
    actual == expected || actual.strip_prefix('/') == Some(expected)
}
