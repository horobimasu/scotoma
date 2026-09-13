use shared::encode;

pub fn encode_key(data: &[u8], char_set: &[u8]) -> String {
    encode::hex_custom_encode(data, char_set)
        .to_uppercase()
        .chars()
        .collect::<Vec<char>>()
        .chunks(100)
        .map(|chunk| {
            let str = chunk
                .iter()
                .collect::<String>();

            format!("{:0<100}", str)
        })
        .collect::<Vec<String>>()
        .join("\n")
}
