//! Bounded, allocation-free protobuf field iteration for the active Run wire.
//! Unknown field numbers are compatible; malformed encodings are not.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WireError(pub &'static str);

impl fmt::Display for WireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for WireError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Field<'a> {
    pub number: u32,
    pub wire: u8,
    pub bytes: &'a [u8],
    pub varint: u64,
}

pub(crate) struct Fields<'a> {
    remaining: &'a [u8],
}

pub(crate) fn fields(bytes: &[u8]) -> Fields<'_> {
    Fields { remaining: bytes }
}

fn varint(bytes: &mut &[u8]) -> Result<u64, WireError> {
    let mut value = 0u64;
    for index in 0..10 {
        let (&byte, rest) = bytes.split_first().ok_or(WireError("truncated varint"))?;
        *bytes = rest;
        if index == 9 && byte > 1 {
            return Err(WireError("varint overflow"));
        }
        value |= u64::from(byte & 127) << (index * 7);
        if byte & 128 == 0 {
            return Ok(value);
        }
    }
    Err(WireError("varint overflow"))
}

fn take<'a>(bytes: &mut &'a [u8], length: usize) -> Result<&'a [u8], WireError> {
    if length > bytes.len() {
        return Err(WireError("truncated field"));
    }
    let (value, rest) = bytes.split_at(length);
    *bytes = rest;
    Ok(value)
}

fn next_field<'a>(bytes: &mut &'a [u8]) -> Result<Field<'a>, WireError> {
    let tag = varint(bytes)?;
    let number = tag >> 3;
    if number == 0 || number > 0x1fff_ffff {
        return Err(WireError("invalid protobuf field number"));
    }
    let mut field = Field {
        number: number as u32,
        wire: (tag & 7) as u8,
        bytes: &[],
        varint: 0,
    };
    match field.wire {
        0 => field.varint = varint(bytes)?,
        1 => field.bytes = take(bytes, 8)?,
        2 => {
            let length =
                usize::try_from(varint(bytes)?).map_err(|_| WireError("field length overflow"))?;
            field.bytes = take(bytes, length)?;
        }
        5 => field.bytes = take(bytes, 4)?,
        // The Run schema has no deprecated groups. Unknown numbers using the
        // four supported encodings above are skipped safely by consumers.
        _ => return Err(WireError("unsupported protobuf wire encoding")),
    }
    Ok(field)
}

impl<'a> Iterator for Fields<'a> {
    type Item = Result<Field<'a>, WireError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }
        let result = next_field(&mut self.remaining);
        if result.is_err() {
            self.remaining = &[];
        }
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_kv_wire_rejects_named_malformed_corpus() {
        for (name, bytes) in [
            ("field zero", vec![0, 0]),
            ("tag truncation", vec![128]),
            (
                "varint overflow",
                vec![8, 255, 255, 255, 255, 255, 255, 255, 255, 255, 2],
            ),
            ("length truncation", vec![10, 4, 8]),
            ("invalid encoding", vec![15]),
            ("fixed64 truncation", vec![9, 1]),
            ("fixed32 truncation", vec![13, 1]),
        ] {
            let mut parsed = fields(&bytes);
            assert!(parsed.next().unwrap().is_err(), "{name}");
            assert!(parsed.next().is_none(), "error terminates parser: {name}");
        }
    }

    #[test]
    fn cursor_kv_wire_preserves_unknown_supported_encodings() {
        let bytes = [
            8, 42, 17, 0, 0, 0, 0, 0, 0, 0, 0, 26, 2, b'o', b'k', 37, 0, 0, 0, 0,
        ];
        let values = fields(&bytes).collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(
            values
                .iter()
                .map(|f| (f.number, f.wire))
                .collect::<Vec<_>>(),
            vec![(1, 0), (2, 1), (3, 2), (4, 5)]
        );
        assert_eq!(values[0].varint, 42);
        assert_eq!(values[2].bytes, b"ok");
        assert!(fields(&[]).next().is_none());
    }

    #[test]
    fn cursor_kv_wire_does_not_hide_trailing_corruption() {
        let mut values = fields(&[8, 1, 128]);
        assert_eq!(values.next().unwrap().unwrap().varint, 1);
        assert!(values.next().unwrap().is_err());
        assert!(values.next().is_none());
    }

    #[test]
    fn cursor_kv_wire_accepts_full_u64_and_checks_tag_range() {
        let max = [8, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1];
        assert_eq!(fields(&max).next().unwrap().unwrap().varint, u64::MAX);
        assert!(fields(&[128, 128, 128, 128, 16, 0])
            .next()
            .unwrap()
            .is_err());
    }
}
