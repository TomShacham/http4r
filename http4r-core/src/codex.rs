use std::io::{Read, Write};
use crate::http_message::CompressionAlgorithm;
use flate2::bufread::{DeflateEncoder, GzEncoder};
use flate2::{Compression, GzBuilder};
use flate2::read::{GzDecoder};
use flate2::write::DeflateDecoder;

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
            CompressionAlgorithm::NONE => {}
        }
    }

    pub fn decode(reader: &mut [u8], mut writer: &mut Vec<u8>, compression: &CompressionAlgorithm) {
        match compression {
            CompressionAlgorithm::GZIP => {
                let mut gzip_decoder = GzDecoder::new(&reader[..]);
                gzip_decoder.read_to_end(&mut writer).unwrap();
            }
            CompressionAlgorithm::DEFLATE => {
                let mut deflater = DeflateDecoder::new(writer);
                deflater.write(&reader[..]).unwrap();
                writer = deflater.finish().unwrap();
            }
            CompressionAlgorithm::NONE => {}
        }

    }
}