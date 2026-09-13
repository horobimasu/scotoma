use std::error::Error;

pub fn hex_custom_encode(data: &[u8], char_set: &[u8]) -> String {
    data
        .iter()
        .flat_map(|byte| {
            [
                char_set[(byte >> 4) as usize] as char,
                char_set[(byte & 0xF) as usize] as char,
            ]
        })
        .collect()
}

pub fn hex_custom_decode(data: &str, char_set: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    if data.len() % 2 != 0 {
        return Err("invalid data length".into());
    }

    data
        .chars()
        .collect::<Vec<char>>()
        .chunks(2)
        .map(|chunk| {
            let high = char_set.iter().position(|&char| char as char == chunk[0])
                .ok_or_else(|| format!("invalid character: {}", chunk[0]))? as u8;

            let low = char_set.iter().position(|&char| char as char == chunk[1])
                .ok_or_else(|| format!("invalid character: {}", chunk[1]))? as u8;

            Ok((high << 4) | low)
        })
        .collect()
}

#[test]
fn test_encode_decode() {
    const CUSTOM_CHARACTERS: &str = "jhsb0ap5zgn3qr67";
    const DATA_TO_ENCODE: &str = "the data to encode";

    let encoded = hex_custom_encode(DATA_TO_ENCODE.as_bytes(), CUSTOM_CHARACTERS.as_bytes());
    let decoded = hex_custom_decode(&encoded, CUSTOM_CHARACTERS.as_bytes())
        .unwrap();

    let decoded_str = String::from_utf8_lossy(&decoded)
        .into_owned();

    assert_eq!(decoded_str, DATA_TO_ENCODE);
}
