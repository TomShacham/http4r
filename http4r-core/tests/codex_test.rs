mod tests {
    use http4r_core::codex::Codex;
    use http4r_core::http_message::CompressionAlgorithm::{DEFLATE, GZIP};

    #[test]
    fn gzip_and_deflate_encode_and_decode_with_flate2(){
        let original_string = "hello world my baby boo".repeat(200);
        let mut bytestring = original_string.as_bytes();

        let mut gzip_encode_writer = Vec::new();
        let mut gzip_decode_writer = Vec::new();

        Codex::encode(&mut bytestring, &mut gzip_encode_writer, GZIP);
        assert_eq!(gzip_encode_writer.len(), 99); // much shorter than bytestring.len()

        Codex::decode(&mut gzip_encode_writer, &mut gzip_decode_writer, &GZIP);

        assert_eq!(gzip_decode_writer.as_slice(), bytestring);

        let mut deflate_encode_writer = Vec::new();
        let mut deflate_decode_writer = Vec::new();

        Codex::encode(&mut bytestring, &mut deflate_encode_writer, DEFLATE);
        assert_eq!(deflate_encode_writer.len(), 81); // much shorter than bytestring.len()

        Codex::decode(&mut deflate_encode_writer, &mut deflate_decode_writer, &DEFLATE);

        assert_eq!(deflate_decode_writer.as_slice(), bytestring);
    }

}