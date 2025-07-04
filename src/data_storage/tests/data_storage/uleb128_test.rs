use johansen_null_eigenspectra::data_storage::uleb128::{decode, encode, encoded_size};

#[test]
fn test_uleb128_basic_values() {
    let test_cases = vec![
        (0, vec![0x00]),
        (1, vec![0x01]),
        (127, vec![0x7F]),
        (128, vec![0x80, 0x01]),
        (255, vec![0xFF, 0x01]),
        (256, vec![0x80, 0x02]),
        (16383, vec![0xFF, 0x7F]),
        (16384, vec![0x80, 0x80, 0x01]),
    ];

    for (value, expected_bytes) in test_cases {
        // Test encoding
        let encoded = encode(value);
        assert_eq!(
            encoded, expected_bytes,
            "Encoding mismatch for value {}",
            value
        );

        // Test decoding
        let (decoded, bytes_used) =
            decode(&encoded).expect(&format!("Failed to decode value {}", value));
        assert_eq!(decoded, value, "Decoding mismatch for value {}", value);
        assert_eq!(
            bytes_used,
            encoded.len(),
            "Byte count mismatch for value {}",
            value
        );

        // Test size calculation
        let calculated_size = encoded_size(value);
        assert_eq!(
            calculated_size,
            encoded.len(),
            "Size calculation mismatch for value {}",
            value
        );
    }
}

#[test]
fn test_uleb128_large_values() {
    let large_values = vec![
        65535,      // 2 bytes
        65536,      // 3 bytes
        2097151,    // 3 bytes
        2097152,    // 4 bytes
        268435455,  // 4 bytes
        268435456,  // 5 bytes
        4294967295, // max u32, 5 bytes
    ];

    for value in large_values {
        let encoded = encode(value);
        let (decoded, bytes_used) =
            decode(&encoded).expect(&format!("Failed to decode large value {}", value));

        assert_eq!(
            decoded, value,
            "Large value encoding/decoding failed for {}",
            value
        );
        assert_eq!(bytes_used, encoded.len());
        assert!(
            encoded.len() <= 5,
            "ULEB128 encoding too long for u32: {} bytes",
            encoded.len()
        );
    }
}

#[test]
fn test_uleb128_error_cases() {
    // Test incomplete encoding
    let incomplete = vec![0x80]; // Has continuation bit but no more bytes
    assert!(decode(&incomplete).is_err());

    // Test empty slice
    let empty = vec![];
    assert!(decode(&empty).is_err());
}
