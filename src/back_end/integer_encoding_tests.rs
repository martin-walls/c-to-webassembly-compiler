#[cfg(test)]
mod integer_encoding_tests {
    use super::super::encode_unsigned_int;

    #[test]
    fn unsigned_int() {
        let result = encode_unsigned_int(65000);
        assert_eq!(result, vec![0b11101000, 0b11111011, 0b00000011]);
    }

    #[test]
    fn unsigned_zero() {
        let result = encode_unsigned_int(0);
        assert_eq!(result, vec![0]);
    }

    #[test]
    fn unsigned_single_byte() {
        let result = encode_unsigned_int(81);
        assert_eq!(result, vec![0b0101_0001]);
    }
}
