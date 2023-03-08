use std::{
    fs::File,
    io::{Read, Result},
};

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;

pub struct TwoDA {
    read_data: Vec<Vec<u8>>,
}

impl TwoDA {
    const CHUNK_SIZE: usize = 0x4000;

    pub fn new(filename: &str) -> TwoDA {
        let data = TwoDA::open_file(filename).unwrap();
        TwoDA { read_data: data }
    }

    fn open_file(filename: &str) -> Result<Vec<Vec<u8>>> {
        let mut file = match File::open(filename) {
            Ok(file) => file,
            Err(_) => panic!("Could not open the file named {}.", filename),
        };

        let mut list_of_chunks: Vec<Vec<u8>> = Vec::new();

        loop {
            let mut chunk = Vec::with_capacity(TwoDA::CHUNK_SIZE);
            let read = file
                .by_ref()
                .take(TwoDA::CHUNK_SIZE as u64)
                .read_to_end(&mut chunk)?;

            if read == 0 {
                break;
            }
            list_of_chunks.push(chunk);
            if read < TwoDA::CHUNK_SIZE {
                break;
            }
        }

        Ok(list_of_chunks)
    }

    pub fn get_rows_and_column(&self, filename: &str) -> HashMap<usize, String> {
        let path = Path::new(filename);
        let file = File::open(path).unwrap();
        let mut reader = BufReader::new(file);

        // Read header
        let mut header_packed = [0u8; 9];
        reader.read_exact(&mut header_packed).unwrap();
        let header = String::from_utf8_lossy(&header_packed);
        if header.trim() != "2DA V2.b\n" {
            println!("No header located.");
            return HashMap::new();
        }

        // Read entire 2da file as bytes
        let mut twoda = Vec::new();
        reader.read_to_end(&mut twoda).unwrap();

        // Find null position
        let mut the_null_pos = None;
        for (idx, b) in twoda.iter().enumerate() {
            if *b == 0 {
                the_null_pos = Some(idx);
                break;
            }
        }
        let the_null_pos = the_null_pos.unwrap();

        // Read number of rows
        let num_of_rows_packed = &twoda[the_null_pos + 1..the_null_pos + 5];
        let num_of_rows = u32::from_le_bytes(num_of_rows_packed.try_into().unwrap()) as usize;

        // Find number of columns
        let mut tab_cnt = 0;
        for (i, b) in twoda.iter().enumerate() {
            if *b == b'\t' && i < the_null_pos {
                tab_cnt += 1;
            }
        }
        let num_of_cols = tab_cnt;

        // Find position after row names
        let mut count = 0;
        let mut after_rownames_pos = 0;
        for (i, b) in twoda.iter().enumerate() {
            if *b == b'\t' && i > the_null_pos {
                if count == num_of_rows - 1 {
                    after_rownames_pos = i;
                    break;
                }
                count += 1;
            }
        }

        // Find number of pointers and data area size
        let num_of_pointers = num_of_rows * num_of_cols;
        let data_area = after_rownames_pos + (num_of_pointers * 2) + 2;

        // Read row labels and values
        let mut row_to_label = HashMap::new();
        let mut row = 0;
        for i in (0..num_of_pointers).step_by(num_of_cols) {
            let pointer_packed =
                &twoda[after_rownames_pos + (i * 2)..after_rownames_pos + ((i + 1) * 2)];
            let pointer = u16::from_le_bytes(pointer_packed.try_into().unwrap()) as i64;
            let value = {
                reader.seek(SeekFrom::Start(data_area as u64)).unwrap();
                reader.seek(SeekFrom::Current(pointer)).unwrap();
                let mut buf = String::new();
                reader.read_line(&mut buf).unwrap();
                buf.trim().to_string()
            };
            if value.is_empty() {
                row += 1;
                continue;
            }
            row_to_label.insert(row, value);
            row += 1;
        }
        row_to_label
    }
}
