extern crate dotenv;
extern crate rand;

use rand::Rng;
use rubato::{Resampler, FastFixedOut };



use std::{env, vec};
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::path::PathBuf;
use std::time::Instant;
use tokio::task::JoinSet;
use tokio::fs::File as TokioFile;
use tokio::io::{AsyncWriteExt, BufWriter};


const BYTE_PER_SAMPLE: usize = 8;
#[tokio::main]
async fn main() {
    let channels = 2;
    // let fs_out = 16000;
    let fs_in = 44100;

    dotenv::dotenv().ok(); // load.env file


    // Open the input
    let default_path = "/Users/dieudonn/Downloads/large-sample-usa.raw";
    let input_path = env::var("LARGE_WAV_PATH").unwrap_or(default_path.to_string());

    let file_in_disk = File::open(input_path).expect("Can't open file");
    let mut file_in_reader = BufReader::new(file_in_disk);
    let indata = read_file(&mut file_in_reader, channels);
    let nbr_input_frames = indata[0].len();


    // Create buffer for storing output
    let num_buffers = 50;

    let duration_total = Instant::now();
    let mut write_files_futures = JoinSet::new();

    for index in 0..num_buffers {
        let indata_copy = indata.clone();
        let _duration = Instant::now();
        let output_filename = format!("output/output-rubato-{}.raw", index);
        let output_path = env::current_dir().unwrap().join(output_filename) ;
        let random_sample_rate = choose_random_sample_rate();
        let ratio = random_sample_rate / 44100_f64;

        // Create buffer for storing output
        let mut outdata = vec![
            Vec::with_capacity(
                2 * (nbr_input_frames as f32 * random_sample_rate as f32 / fs_in as f32) as usize
            );
            channels
        ];


        // Instanciate the re-sampler 
        let params = rubato::PolynomialDegree::Septic;
        println!("One file will have ratio sample {}", ratio);
        let mut resampler = FastFixedOut::<f64>::new( 
            ratio,
            2.0,
            params,
            1024,
            2,
        ).unwrap();

        // resampler.set_resample_ratio_relative(0.8, false);

        let mut input_frames_next = resampler.input_frames_next();
        let resampler_delay = resampler.output_delay();
        let mut outbuffer = vec![vec![0.0f64; resampler.output_frames_max()]; channels];
        let mut indata_slices: Vec<&[f64]> = indata_copy.iter().map(|v| &v[..]).collect();

        // Process all full chunks
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

        let nbr_output_frames = (nbr_input_frames as f32 * random_sample_rate as f32 / fs_in as f32) as usize;
        // println!("Re-Sample was done in {:?}.. write file to disk now", duration.elapsed());
        let future = write_frames(
            outdata,
            output_path,
            resampler_delay,
            nbr_output_frames,
        );
        write_files_futures.spawn(future);
    }
    println!("Re-Sample of 50 file was done in {:?}.. write file to disk now", duration_total.elapsed());
    while let Some(res) = write_files_futures.join_next().await {
        let _out = res;
        println!("wrote 1 file to disk");
    }
}


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
    output_path: PathBuf,
    frames_to_skip: usize,
    frames_to_write: usize,
) {
    let mut file_out_disk = BufWriter::new(TokioFile::create(output_path).await.unwrap());

    let channels = waves.len();
    // println!("We have {}  elements in the vec to export now", frames_to_write);
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


fn choose_random_sample_rate() -> f64 {
    let sample_rates: Vec<f64> = vec![8000_f64, 16000_f64, 32000_f64, 44100_f64, 48000_f64];
    let random_index = rand::thread_rng().gen_range(0..sample_rates.len());
    sample_rates[random_index]
}
