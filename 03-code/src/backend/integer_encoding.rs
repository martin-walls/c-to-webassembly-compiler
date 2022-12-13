#[cfg(test)]
#[path = "integer_encoding_tests.rs"]
mod integer_encoding_tests;

pub fn encode_unsigned_int(mut value: u128) -> Vec<u8> {
    let mut bytes = Vec::new();

    loop {
        // take lowest 7 bits
        let mut byte: u8 = (value & 0b0111_1111) as u8;
        value >>= 7;
        if value != 0 {
            // there are more bits left to process
            // add a 1 to the front of every byte except the last one (the MSB)
            byte |= 0b1000_0000
        }
        bytes.push(byte);
        if value == 0 {
            break;
        }
    }

    bytes
}

pub fn encode_signed_int(value: i128) -> Vec<u8> {
    todo!("LEB128 signed integer encoding")
}
