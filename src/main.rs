use std::fs::File;
use std::io::Read;
use std::io::Cursor;
use byteorder::{ReadBytesExt, LittleEndian};

#[derive(Debug)]
struct WaveFile {
    chunk_id: [u8; 4],
    chunk_size: u32,
    format: [u8; 4],
    subchunk1_id: [u8; 4],
    subchunk1_size: u32,
    audio_format: u16,
    num_channels: u16,
    sample_rate: u32,
    byte_rate: u32,
    block_align: u16,
    bits_per_sample: u16,
    subchunk2_id: [u8; 4],
    subchunk2_size: u32,
    data: Vec<u8>,
}

fn main() {
    let mut f = File::open("C:/Users/tabu1/OneDrive/Rust Projects/audio_compressor/src/test.wav").unwrap();
    let mut buffer = Vec::new();

    // read the whole file
    f.read_to_end(&mut buffer).unwrap();

    let mut cursor = Cursor::new(buffer);
    let mut wave_file = WaveFile {
        chunk_id: read_array(&mut cursor, 4),
        chunk_size: cursor.read_u32::<LittleEndian>().unwrap(),
        format: read_array(&mut cursor, 4),
        subchunk1_id: [0; 4],
        subchunk1_size: 0,
        audio_format: 0,
        num_channels: 0,
        sample_rate: 0,
        byte_rate: 0,
        block_align: 0,
        bits_per_sample: 0,
        subchunk2_id: [0; 4],
        subchunk2_size: 0,
        data: Vec::new(),
    };

    loop {
        let chunk_id = read_array(&mut cursor, 4);
        let chunk_size = cursor.read_u32::<LittleEndian>().unwrap();
        match &chunk_id {
            b"JUNK" => {
                // Skip the JUNK chunk.
                cursor.set_position(cursor.position() + chunk_size as u64);
            }
            b"fmt " => {
                wave_file.subchunk1_id = chunk_id;
                wave_file.subchunk1_size = chunk_size;
                wave_file.audio_format = cursor.read_u16::<LittleEndian>().unwrap();
                wave_file.num_channels = cursor.read_u16::<LittleEndian>().unwrap();
                wave_file.sample_rate = cursor.read_u32::<LittleEndian>().unwrap();
                wave_file.byte_rate = cursor.read_u32::<LittleEndian>().unwrap();
                wave_file.block_align = cursor.read_u16::<LittleEndian>().unwrap();
                wave_file.bits_per_sample = cursor.read_u16::<LittleEndian>().unwrap();
            }
            b"data" => {
                wave_file.subchunk2_id = chunk_id;
                wave_file.subchunk2_size = chunk_size;
                // wave_file.data = vec![0; chunk_size as usize];
                cursor.read_exact(&mut wave_file.data).unwrap();
                break;
            }
            _ => panic!("unexpected chunk"),
        }
    }

    println!("{:#?}", wave_file);
}

fn compress(inout: WaveFile) {
    // do something with input.data
}
