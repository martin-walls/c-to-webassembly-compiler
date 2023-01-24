pub fn encode_float(value: f64) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}
