use super::*;

#[test]
fn decodes_base64_response() {
    let mut buf = EncodedBytes::new("application/grpc-web-text+proto").unwrap();

    buf.append(Bytes::from_static(b"AAAAAAA=")).unwrap();

    assert_eq!(&buf[..], &[0, 0, 0, 0, 0]);
    assert!(buf.raw_buf.is_empty());
}

#[test]
fn buffers_incomplete_base64_quartets() {
    let mut buf = EncodedBytes::new("application/grpc-web-text+proto").unwrap();

    buf.append(Bytes::from_static(b"YWJjZA=")).unwrap();
    assert_eq!(&buf[..], b"abc");
    assert_eq!(&buf.raw_buf[..], b"ZA=");

    // Consuming decoded data must not affect the pending encoded bytes.
    assert_eq!(&buf.take(3)[..], b"abc");
    buf.append(Bytes::from_static(b"=")).unwrap();
    assert_eq!(&buf[..], b"d");
    assert!(buf.raw_buf.is_empty());
}

#[test]
fn decodes_concatenated_base64_segments_at_every_chunk_size() {
    // Include two-byte padding, one-byte padding, and an unpadded segment.
    let encoded = b"YQ==YmM=ZGVmZ2hpag==";

    for chunk_size in 1..=encoded.len() {
        let mut buf = EncodedBytes::new("application/grpc-web-text+proto").unwrap();
        for chunk in encoded.chunks(chunk_size) {
            buf.append(Bytes::copy_from_slice(chunk)).unwrap();
        }

        assert_eq!(&buf[..], b"abcdefghij", "chunk size {chunk_size}");
        assert!(buf.raw_buf.is_empty());
    }
}

#[test]
fn rejects_invalid_base64() {
    for encoded in [b"!!!!", b"AA=A", b"A==="] {
        let mut buf = EncodedBytes::new("application/grpc-web-text+proto").unwrap();
        assert!(matches!(
            buf.append(Bytes::copy_from_slice(encoded)),
            Err(Error::Base64DecodeError(_))
        ));
    }
}

#[test]
fn binary_response_is_not_base64_decoded() {
    let mut buf = EncodedBytes::new("application/grpc-web+proto").unwrap();
    let bytes = Bytes::from_static(b"\x00\x80!!!!");

    buf.append(bytes.clone()).unwrap();

    assert_eq!(&buf[..], &bytes[..]);
    assert!(buf.raw_buf.is_empty());
}
