extern crate rubato;
use rubato::{
    implement_resampler, FastFixedOut, PolynomialDegree,
};
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::task::JoinSet;
use std::convert::TryInto;

use std::{env, vec};
use rand::Rng;
use std::fs::File;
use tokio::fs::{File as TokioFile};
use std::io::prelude::{Read, Seek};
use std::io::BufReader;
use std::time::Instant;

const BYTE_PER_SAMPLE: usize = 8;

implement_resampler!(SliceResampler, &[&[T]], &mut [Vec<T>]);

/// Helper to read an entire file to memory
fn read_file<R: Read + Seek>(inbuffer: &mut R, channels: usize) -> Vec<Vec<f64>> {
    let mut buffer = vec![0u8; BYTE_PER_SAMPLE];
    let mut wfs = Vec::with_capacity(channels);
    for _chan in 0..channels {
        wfs.push(Vec::new());
    }
    'outer: loop {
        for wf in wfs.iter_mut() {
            let bytes_read = inbuffer.read(&mut buffer).unwrap();
            if bytes_read == 0 {
                break 'outer;
            }
            let value = f64::from_le_bytes(buffer.as_slice().try_into().unwrap());
            //idx += 8;
            wf.push(value);
        }
    }
    wfs
}

/// Helper to write all frames to a file
async fn write_frames(
    waves: Vec<Vec<f64>>,
    output: String,
    frames_to_skip: usize,
    frames_to_write: usize,
) {
    let mut file_out_disk = BufWriter::new(TokioFile::create(output).await.unwrap());    
    let channels = waves.len();
    let end = frames_to_skip + frames_to_write;
    for frame in frames_to_skip..end {
        for wave in waves.iter().take(channels) {
            let value64 = wave[frame];
            let bytes = value64.to_le_bytes();
            file_out_disk.write_all(&bytes).await.unwrap();
        }
    }
}

fn append_frames(buffers: &mut [Vec<f64>], additional: &[Vec<f64>], nbr_frames: usize) {
    buffers
        .iter_mut()
        .zip(additional.iter())
        .for_each(|(b, a)| b.extend_from_slice(&a[..nbr_frames]));
}
#[tokio::main]
async fn main() {

    let mut write_files_futures = JoinSet::new();
    let num_buffers = 50;


    let file_in = "/Users/dieudonn/Downloads/large-sample-usa.raw";
    let fs_in_str = "44100";
    
    let fs_in = fs_in_str.parse::<usize>().unwrap();
    
    let channels_str = "2";
    let channels = channels_str.parse::<usize>().unwrap();

    println!("Copy input file to buffer");
    let file_in_disk = File::open(file_in).expect("Can't open file");
    let mut file_in_reader = BufReader::new(file_in_disk);
    let indata = read_file(&mut file_in_reader, channels);
    let nbr_input_frames = indata[0].len();

    let duration_total = Instant::now();

    for index in 0..num_buffers {
        let output_filename = format!("output/output-rubato-{}.raw", index);
        let output_path = env::current_dir().unwrap().join(output_filename);
        let file_out = output_path.to_str().unwrap().to_string();
        let fs_out_str = choose_random_sample_rate();
        let fs_out = fs_out_str.parse::<usize>().unwrap();
        // println!("Sample {} for output {}", &fs_out,&file_out);

        // Create buffer for storing output
        let mut outdata = vec![
            Vec::with_capacity(
                2 * (nbr_input_frames as f32 * fs_out as f32 / fs_in as f32) as usize
            );
            channels
        ];

        let f_ratio = fs_out as f64 / fs_in as f64;
        

        // Create resampler
        let mut resampler=  FastFixedOut::<f64>::new(f_ratio, 1.1, PolynomialDegree::Septic, 1024, channels).unwrap();

        // Prepare
        let mut input_frames_next = resampler.input_frames_next();
        let resampler_delay = resampler.output_delay();
        let mut outbuffer = vec![vec![0.0f64; resampler.output_frames_max()]; channels];
        let mut indata_slices: Vec<&[f64]> = indata.iter().map(|v| &v[..]).collect();

        // Process all full chunks
        let start = Instant::now();

        while indata_slices[0].len() >= input_frames_next {
            let (nbr_in, nbr_out) = resampler
                .process_into_buffer(&indata_slices, &mut outbuffer, None)
                .unwrap();
            for chan in indata_slices.iter_mut() {
                *chan = &chan[nbr_in..];
            }
            append_frames(&mut outdata, &outbuffer, nbr_out);
            input_frames_next = resampler.input_frames_next();
        }

        // Process a partial chunk with the last frames.
        if !indata_slices[0].is_empty() {
            let (_nbr_in, nbr_out) = resampler
                .process_partial_into_buffer(Some(&indata_slices), &mut outbuffer, None)
                .unwrap();
            append_frames(&mut outdata, &outbuffer, nbr_out);
        }

        let duration = start.elapsed();
        println!("Resampling took: {:?}", duration);

        let nbr_output_frames = (nbr_input_frames as f32 * fs_out as f32 / fs_in as f32) as usize;
        // println!(
        //     "Processed {} input frames into {} output frames",
        //     nbr_input_frames, nbr_output_frames
        // );

        // Write output to file, trimming off the silent frames from both ends.
        let future = write_frames(
            outdata,
            file_out,
            resampler_delay,
            nbr_output_frames,
        );
        write_files_futures.spawn(future);
    }
    while let Some(_) = write_files_futures.join_next().await {

    }
    let duration_total_time = duration_total.elapsed();
    println!("Resampling 50 files took: {:?}", duration_total_time);
}


fn choose_random_sample_rate() -> &'static str {
    let sample_rates: Vec<&str> = vec![ "8000", "16000", "32000"];
    let random_index = rand::thread_rng().gen_range(0..sample_rates.len());
    sample_rates[random_index]
}
