use johansen_null_eigenspectra::data_storage::uleb128::{
    Uleb128Error, decode, encode, encoded_size,
};

#[test]
fn test_encode_basic_values() {
    assert_eq!(encode(0), vec![0x00]);
    assert_eq!(encode(1), vec![0x01]);
    assert_eq!(encode(127), vec![0x7F]);
    assert_eq!(encode(128), vec![0x80, 0x01]);
    assert_eq!(encode(255), vec![0xFF, 0x01]);
    assert_eq!(encode(256), vec![0x80, 0x02]);
    assert_eq!(encode(300), vec![0xAC, 0x02]);
    assert_eq!(encode(16383), vec![0xFF, 0x7F]);
    assert_eq!(encode(16384), vec![0x80, 0x80, 0x01]);
}

#[test]
fn test_decode_basic_values() {
    assert_eq!(decode(&[0x00]).unwrap(), (0, 1));
    assert_eq!(decode(&[0x01]).unwrap(), (1, 1));
    assert_eq!(decode(&[0x7F]).unwrap(), (127, 1));
    assert_eq!(decode(&[0x80, 0x01]).unwrap(), (128, 2));
    assert_eq!(decode(&[0xFF, 0x01]).unwrap(), (255, 2));
    assert_eq!(decode(&[0x80, 0x02]).unwrap(), (256, 2));
    assert_eq!(decode(&[0xAC, 0x02]).unwrap(), (300, 2));
    assert_eq!(decode(&[0xFF, 0x7F]).unwrap(), (16383, 2));
    assert_eq!(decode(&[0x80, 0x80, 0x01]).unwrap(), (16384, 3));
}

#[test]
fn test_encoded_size() {
    assert_eq!(encoded_size(0), 1);
    assert_eq!(encoded_size(127), 1);
    assert_eq!(encoded_size(128), 2);
    assert_eq!(encoded_size(16383), 2);
    assert_eq!(encoded_size(16384), 3);
    assert_eq!(encoded_size(2097151), 3);
    assert_eq!(encoded_size(2097152), 4);
    assert_eq!(encoded_size(268435455), 4);
    assert_eq!(encoded_size(268435456), 5);
    assert_eq!(encoded_size(u32::MAX), 5);
}

#[test]
fn test_roundtrip() {
    let test_values = vec![
        0,
        1,
        127,
        128,
        255,
        256,
        16383,
        16384,
        65535,
        65536,
        2097151,
        2097152,
        268435455,
        268435456,
        u32::MAX,
    ];

    for value in test_values {
        let encoded = encode(value);
        let (decoded, bytes_used) = decode(&encoded).unwrap();
        assert_eq!(decoded, value);
        assert_eq!(bytes_used, encoded.len());
        assert_eq!(encoded_size(value), encoded.len());
    }
}

#[test]
fn test_error_cases() {
    // 測試不完整的編碼
    assert_eq!(decode(&[0x80]), Err(Uleb128Error::IncompleteEncoding));

    // 測試空切片
    assert_eq!(decode(&[]), Err(Uleb128Error::IncompleteEncoding));

    // 測試 u32::MAX 的正確編碼應該成功
    let max_u32_encoded = encode(u32::MAX);
    let result = decode(&max_u32_encoded);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().0, u32::MAX);

    // 測試會超過 5 個位元組限制的編碼
    let too_long = vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x80];
    assert_eq!(decode(&too_long), Err(Uleb128Error::EncodingTooLong));
}
