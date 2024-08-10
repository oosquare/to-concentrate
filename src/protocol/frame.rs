use bytes::{Buf, BufMut};
use serde_json::Error as SerdeError;
use snafu::prelude::*;

use crate::protocol::data::Protocol;

/// A wrapper of [`Protocol`] for converting the internal data from and to
/// bytes and being transmitted through byte stream.
///
/// The layout of a [`Frame`] in bytes is described below:
/// - starts with a `b'+'` and a `u64` as inner data's length,
/// - followed by data of the length mentioned above.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    data: Protocol,
}

impl Frame {
    /// Parse a [`Frame`] from one of buf's prefix and advance buf's cursor.
    /// Return a [`Frame`] and the offset from the initial position.
    ///
    /// Note that the cursor could be advanced even if it fails to parse a
    /// [`Frame`], its final position is expected to be valid only when it
    /// succeeds.
    ///
    /// # Errors
    ///
    /// This function will return an error if there is no enough byte or the
    /// data is broken.
    pub fn parse<B: Buf>(mut buf: B) -> Result<(Self, usize), ParseFrameError> {
        // Try to get `b'+'`.
        ensure!(buf.remaining() >= 1, IncompleteSnafu);
        ensure!(buf.get_u8() == b'+', InvalidStartSnafu);

        // Try to get the length.
        ensure!(buf.remaining() >= 8, IncompleteSnafu);
        let len = buf.get_u64() as usize;
        ensure!(len > 0, InvalidLengthSnafu);

        // Try to parse a `Frame` from remaining bytes.
        ensure!(buf.remaining() >= len, IncompleteSnafu);
        let reader = buf.take(len).reader();
        let data: Protocol = serde_json::from_reader(reader).context(DeserializationSnafu)?;

        Ok((data.into(), 9 + len))
    }

    /// Serialize a [`Frame`] and write it to buf.
    ///
    /// # Errors
    ///
    /// This function will return an error if the serialization fails.
    pub fn write<B: BufMut>(&self, mut buf: B) -> Result<(), WriteFrameError> {
        let data = serde_json::to_string(&self.data).context(SerializationSnafu)?;
        buf.put_u8(b'+');
        buf.put_u64(data.len() as u64);
        buf.put_slice(data.as_bytes());
        Ok(())
    }
}

impl From<Protocol> for Frame {
    fn from(value: Protocol) -> Self {
        Self { data: value }
    }
}

impl From<Frame> for Protocol {
    fn from(value: Frame) -> Self {
        value.data
    }
}

/// An error type for parsing a [`Frame`] from bytes.
#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum ParseFrameError {
    #[snafu(display("Could not parse a frame with incomplete data"))]
    Incomplete,
    #[snafu(display("Could not parse the start symbol"))]
    InvalidStart,
    #[snafu(display("The content length should be non-zero"))]
    InvalidLength,
    #[snafu(display("Could not deserialize data"))]
    Deserialization { source: SerdeError },
}

/// An error type for writing a [`Frame`] to bytes.
#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum WriteFrameError {
    #[snafu(display("Could not serialize frame"))]
    Serialization { source: SerdeError },
}

#[cfg(test)]
mod tests {
    use super::*;

    use bytes::BytesMut;
    use tokio::time::Duration;

    use crate::protocol::data::Response;

    #[test]
    fn frame_parse() {
        let inner = br#"
            {
                "type": "Response",
                "method": "Query",
                "stage": "Preparation",
                "total": {
                    "secs": 20,
                    "nanos": 0
                },
                "remaining": {
                    "secs": 15,
                    "nanos": 0
                },
                "past": {
                    "secs": 5,
                    "nanos": 0
                }
            }
        "#;
        let mut raw = BytesMut::new();
        raw.put_u8(b'+');
        raw.put_u64(inner.len() as u64);
        raw.put_slice(inner);
        raw.put_slice(b"whatever");

        let (actual, offset) = Frame::parse(&mut raw).unwrap();

        let expected = Protocol::Response(Response::Query {
            stage: "Preparation".to_owned(),
            total: Duration::from_secs(20),
            remaining: Duration::from_secs(15),
            past: Duration::from_secs(5),
        })
        .into();

        assert_eq!(actual, expected);
        assert_eq!(offset, 9 + inner.len());

        assert_eq!(raw.as_ref(), b"whatever");
    }

    #[test]
    fn frame_parse_error_incomplete() {
        let mut raw = BytesMut::new();
        assert!(matches!(
            Frame::parse(&mut raw),
            Err(ParseFrameError::Incomplete),
        ));

        let mut raw = BytesMut::new();
        raw.put_u8(b'+');
        raw.put_u64(10);
        assert!(matches!(
            Frame::parse(&mut raw),
            Err(ParseFrameError::Incomplete),
        ));

        let mut raw = BytesMut::new();
        raw.put_u8(b'+');
        raw.put_u64(20);
        raw.put_slice(b"not enough");
        assert!(matches!(
            Frame::parse(&mut raw),
            Err(ParseFrameError::Incomplete),
        ));
    }

    #[test]
    fn frame_parse_error_invalid_start() {
        let mut raw = BytesMut::from(&b"?"[..]);
        assert!(matches!(
            Frame::parse(&mut raw),
            Err(ParseFrameError::InvalidStart),
        ));
    }

    #[test]
    fn frame_parse_error_invalid_length() {
        let mut raw = BytesMut::new();
        raw.put_u8(b'+');
        raw.put_u64(0);
        assert!(matches!(
            Frame::parse(&mut raw),
            Err(ParseFrameError::InvalidLength),
        ));
    }

    #[test]
    fn frame_parse_error_deserialization() {
        let mut raw = BytesMut::new();
        raw.put_u8(b'+');
        raw.put_u64(8);
        raw.put_slice(b"whatever");
        assert!(matches!(
            Frame::parse(&mut raw),
            Err(ParseFrameError::Deserialization { .. }),
        ));
    }
}
