use std::io::{Read, Write};
use crate::http_message::CompressionAlgorithm;
use flate2::bufread::{DeflateEncoder, GzEncoder};
use flate2::{Compression};
use flate2::read::{GzDecoder, DeflateDecoder};

pub struct Codex {}
impl Codex {
    pub fn encode(reader: &[u8], mut writer: &mut Vec<u8>, compression: CompressionAlgorithm) {
        match compression {
            CompressionAlgorithm::GZIP => {
                let mut gzip_encoder = GzEncoder::new(reader, Compression::fast());
                gzip_encoder.read_to_end(&mut writer).unwrap();
            }
            CompressionAlgorithm::DEFLATE => {
                let mut deflate_encoder = DeflateEncoder::new(reader, Compression::fast());
                deflate_encoder.read_to_end(&mut writer).unwrap();
            }
            CompressionAlgorithm::BROTLI => {
                let mut encoder = brotli::CompressorReader::new(reader, reader.len(), 5, 10);
                encoder.read_to_end(writer).unwrap();
            }
            CompressionAlgorithm::NONE => panic!("Cannot decode with no compression algorithmw")
        }
    }

    pub fn decode(reader: &mut [u8], mut writer: &mut Vec<u8>, compression: &CompressionAlgorithm) {
        match compression {
            CompressionAlgorithm::GZIP => {
                let mut gzip_decoder = GzDecoder::new(&reader[..]);
                gzip_decoder.read_to_end(&mut writer).unwrap();
            }
            CompressionAlgorithm::DEFLATE => {
                let mut deflater = DeflateDecoder::new(&reader[..]);
                deflater.read_to_end(writer).unwrap();
            }
            CompressionAlgorithm::BROTLI => {
                let mut writer = brotli::DecompressorWriter::new(writer, reader.len());
                writer.write(reader).unwrap();
            }
            CompressionAlgorithm::NONE => panic!("Cannot decode with no compression algorithmw")
        }

    }
}