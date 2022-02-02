
#[cfg(test)]
mod tests {
    use http4r_core::codex::Codex;
    use http4r_core::http_message::CompressionAlgorithm::{BROTLI, DEFLATE, GZIP};

    #[test]
    fn gzip_encode_and_decode_with_flate2(){
        let original_string = "hello world my baby boo".repeat(200);
        let mut bytestring = original_string.as_bytes();

        let mut encode_writer = Vec::new();
        let mut decode_writer = Vec::new();

        Codex::encode(&mut bytestring, &mut encode_writer, GZIP);
        assert_eq!(encode_writer.len(), 99); // much shorter than bytestring.len()

        Codex::decode(&mut encode_writer, &mut decode_writer, &GZIP);
        assert_eq!(decode_writer.as_slice(), bytestring);
    }

    #[test]
    fn deflate_encode_and_decode_with_flate2(){
        let original_string = "hello world my baby boo".repeat(200);
        let mut bytestring = original_string.as_bytes();

        let mut encode_writer = Vec::new();
        let mut decode_writer = Vec::new();

        Codex::encode(&mut bytestring, &mut encode_writer, DEFLATE);
        assert_eq!(encode_writer.len(), 81); // much shorter than bytestring.len()

        Codex::decode(&mut encode_writer, &mut decode_writer, &DEFLATE);

        assert_eq!(decode_writer.as_slice(), bytestring);
    }

    #[test]
    fn brotli_encode_and_decode(){
        let original_string = "hello world my baby boo".repeat(200);
        let mut bytestring = original_string.as_bytes();

        let mut encode_writer = Vec::new();
        let mut decode_writer = Vec::new();

        Codex::encode(&mut bytestring, &mut encode_writer, BROTLI);
        assert_eq!(encode_writer.len(), 32); // much shorter than bytestring.len()

        Codex::decode(&mut encode_writer, &mut decode_writer, &BROTLI);

        assert_eq!(decode_writer.as_slice(), bytestring);
    }

}