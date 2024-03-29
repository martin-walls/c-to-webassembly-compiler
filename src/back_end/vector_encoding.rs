use crate::back_end::integer_encoding::encode_unsigned_int;
use crate::back_end::to_bytes::ToBytes;

/// Vector is encoded as its length followed by each element in turn
pub fn encode_vector<T: ToBytes>(elements: &Vec<T>) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.append(&mut encode_unsigned_int(elements.len() as u128));
    for element in elements {
        bytes.append(&mut element.to_bytes());
    }
    bytes
}
