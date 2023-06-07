use std::fs::File;
use std::io::Read;
use std::io::Cursor;
use byteorder::{ReadBytesExt, ByteOrder, LittleEndian};
use std::cmp::Ordering;
use num::Float;
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

fn bytes_to_samples(data: &[u8]) -> Vec<i16> {
    let mut samples = Vec::new();
    for i in (0..data.len()).step_by(2) {
        let sample = LittleEndian::read_i16(&data[i..i+2]);
        samples.push(sample);
    }
    samples
}

fn samples_to_bytes(samples: &[i16]) -> Vec<u8> {
    let mut data = Vec::new();
    for &sample in samples {
        let bytes = sample.to_le_bytes();
        data.extend_from_slice(&bytes);
    }
    data
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
                wave_file.data = vec![0; chunk_size as usize];
                cursor.read_exact(&mut wave_file.data).unwrap();
                break;
            }
            _ => panic!("unexpected chunk"),
        }
    }

    println!("{:#?}", wave_file);
}

fn rms(samples: &[f32], length: usize) -> f32 {
    let length = length as f32;
    let sum = samples.iter().map(|&sample| {
        let sample = sample as f32;
        sample * sample
    }).sum::<f32>();
    (sum / length).sqrt()
}

fn percentile_nearest_rank(samples: &Vec<f32>, lookback: usize, threshold: usize) -> f32 {
    let mut sorted_samples = samples.iter().cloned().collect::<Vec<f32>>();
    sorted_samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    let rank = (threshold as f32 / 100.0 * lookback as f32).round() as usize;
    sorted_samples[rank - 1]
}

fn compressor(signal: i16, ratio: f32, threshold: usize, lookback: usize, use_rms: bool, rms_window: usize, knee_type: &str, samples: &Vec<f32>) -> f32 {
    let signal = signal as f32;
    let abs_signal = signal.abs();

    let signal_rms = rms(&samples.iter().map(|&sample| sample.abs()).collect::<Vec<f32>>(), rms_window);
    let signal_percentile = percentile_nearest_rank(&samples, lookback, threshold);
    let signal_percentile_rms = percentile_nearest_rank(&samples.iter().map(|&sample| rms(&vec![sample; rms_window], rms_window)).collect::<Vec<f32>>(), lookback, threshold);

    if use_rms {
        if knee_type == "hard" {
            if abs_signal <= signal_percentile_rms {
                signal
            } else {
                signal_percentile_rms + (signal - signal_percentile_rms) / ratio
            }
        } else {
            if abs_signal <= signal_percentile_rms || signal_percentile_rms == 0.0 {
                signal
            } else {
                let soft_knee_ratio = 1.0 + ((ratio - 1.0) * (abs_signal - signal_percentile_rms)) / abs_signal;
                signal / soft_knee_ratio
            }
        }
    } else {
        if knee_type == "hard" {
            if abs_signal <= signal_percentile {
                signal
            } else {
                signal_percentile + (signal - signal_percentile) / ratio
            }
        } else {
            if abs_signal <= signal_percentile || signal_percentile == 0.0 {
                signal
            } else {
                let soft_knee_ratio = 1.0 + ((ratio - 1.0) * (abs_signal - signal_percentile)) / abs_signal;
                signal / soft_knee_ratio
            }
        }
    }
}

fn read_array<R: Read>(read: &mut R, len: usize) -> [u8; 4] {
    let mut buf = [0; 4];
    read.read_exact(&mut buf[0..len]).unwrap();
    buf
}