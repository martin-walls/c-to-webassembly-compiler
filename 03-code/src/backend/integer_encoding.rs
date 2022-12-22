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

pub fn encode_signed_int(mut value: i128) -> Vec<u8> {
    let mut bytes = Vec::new();

    let mut more_bits = true;

    while more_bits {
        // take lowest 7 bits
        let mut byte: u8 = (value & 0b0111_1111) as u8;
        value >>= 7;

        // sign bit of the byte is the second-highest bit
        // (the highest of the 7 bits we took from value)
        let sign_bit = (byte >> 6) & 1;
        // check if we've reached the end of the number, of either positive or negative number
        if (value == 0 && sign_bit == 0) || (value == -1 && sign_bit == 1) {
            // all bits have been processed
            more_bits = false;
        } else {
            // there are more bits left to process
            // add a 1 to the front of every byte except the last one (the MSB)
            byte |= 0b1000_0000
        }
        bytes.push(byte);
    }

    bytes
}
