use crate::{Error, Result};
use binrw::{binrw, BinRead, BinWrite, BinWriterExt};
use bitvec::prelude::*;
use std::fs::File;
use std::io::{Read, Seek};

#[binrw]
#[brw(little)]
#[derive(Debug, Eq, PartialEq)]
pub struct TpcHeaders {
    data_size: u32,
    reserved: u32,
    x_size: u16,
    y_size: u16,
    encoding: u8,
    dummy: [u8; 115],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Eq, PartialEq)]
pub struct Tpc {
    headers: TpcHeaders,
    #[br(ignore)]
    pixels: Vec<u8>,
}

impl Tpc {
    pub fn new(tpc_filename: &str) -> Self {
        let mut buffer = Self::open_file(tpc_filename).unwrap();

        let mut self_return = Self::read(&mut buffer).unwrap();

        buffer
            .seek(std::io::SeekFrom::Start(128))
            .expect("Not a valid TPC file, not long enough");

        let pixels = if self_return.headers.data_size == 0 && self_return.headers.encoding == 2 {
            Self::calculate_second_encoding_pixels(&self_return, &mut buffer)
        } else if self_return.headers.data_size == 0 && self_return.headers.encoding == 4 {
            Self::calculate_fourth_encoding_pixels(&self_return, &mut buffer)
        } else if self_return.headers.encoding == 2 {
            Self::decode_dxt1(&self_return, &mut buffer)
        } else if self_return.headers.encoding == 4 {
            Self::decode_dxt5(&self_return, &mut buffer)
        } else {
            Err(Error::MissingHeader)
        };

        self_return.pixels = pixels.expect("Not a valid TPC file.");

        return self_return;
    }

    fn open_file(filename: &str) -> Result<File> {
        File::open(filename).map_err(Into::into)
    }

    fn bgr24_to_rgb24(bgr24: u32) -> u32 {
        ((bgr24 & 0xFF) << 16) | (bgr24 & 0xFF00FF00) | ((bgr24 >> 16) & 0xFF)
    }

    fn calculate_second_encoding_pixels(&self, file: &mut File) -> Result<Vec<u8>> {
        for y in (1..self.headers.y_size).rev() {
            let offset = self.headers.x_size * (y - 1);

            let mut packed: Vec<u8> = vec![0; 3 * self.headers.y_size as usize];
            file.read_exact(&mut packed)?;

            let mut bytes = Vec::new();
            for idx in 0..self.headers.y_size {
                let start = idx * 3;
                let end = start + 3;

                for j in start..end {
                    bytes.push(packed[j as usize]);
                }
            }

            let mut bytes2: Vec<u8> = Vec::new();
            let mut outer_idx: u8 = 0;

            for idx in 0..bytes.len() {
                if idx % 3 == 0 && idx > 0 {
                    bytes2.push(0);
                    outer_idx += 1;
                }
                bytes2.push(bytes[idx]);
                outer_idx += 1;
            }
            bytes.insert(outer_idx as usize, 0);
            let mut pixels: Vec<u32> = packed
                .chunks_exact(4)
                .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
                .collect();
            pixels.resize((self.headers.x_size * self.headers.y_size).into(), 0);

            let pixels_offset = offset..offset + self.headers.y_size;

            pixels_offset
                .zip(pixels.clone().into_iter())
                .for_each(|(idx, pixel)| pixels[idx as usize] = pixel);
        }

        Ok(Vec::new()) // todo!
    }
    fn calculate_fourth_encoding_pixels(&self, file: &mut File) -> Result<Vec<u8>> {
        todo!()
    }

    fn decode_dxt1(&self, file: &mut File) -> Result<Vec<u8>> {
        todo!()
    }

    fn decode_dxt5(&self, file: &mut File) -> Result<Vec<u8>> {
        todo!()
    }
}
