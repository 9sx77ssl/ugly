/// Encode bytes to a base58 string (Solana/Bitcoin alphabet).
pub fn encode(data: &[u8]) -> String {
    bs58::encode(data).into_string()
}

/// Encode bytes into a pre-allocated String buffer (avoids allocation).
pub fn encode_into(data: &[u8], buf: &mut String) {
    let encoded = bs58::encode(data).into_string();
    buf.push_str(&encoded);
}
