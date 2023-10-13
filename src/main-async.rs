use rubato::{Resampler, SincFixedIn, SincInterpolationType, SincInterpolationParameters, WindowFunction};
use std::env;
use std::fs::File;
use std::io::{BufReader, Read, Seek, Write};
use std::time::Instant;

use tokio::fs::File as TokyioFile;
use tokio::io::{AsyncWriteExt, Result};

// This file is the equivalent of the main but write in async , it does not work properly , the output does not have the correct size

const BYTE_PER_SAMPLE: usize = 8;

#[tokio::main]
async fn main() -> Result<()> {

    dotenv::dotenv().ok(); // load.env file
    
    let channels = 2;
    let fs_out = 48000;
    let fs_in = 44100;

    let default_path = "/Users/dieudonn/Downloads/large-sample-usa.raw";
    let input_path = env::var("LARGE_WAV_PATH").unwrap_or(default_path.to_string());

    let file_in_disk = File::open(input_path).expect("Can't open file");
    let mut file_in_reader = BufReader::new(file_in_disk);
    let indata = read_file(&mut file_in_reader, channels);
    let nbr_input_frames = indata[0].len();

    let mut outdata = vec![
        Vec::with_capacity(2 * (nbr_input_frames as f32 * fs_out as f32 / fs_in as f32) as usize);
        channels
    ];

    let f_ratio = fs_out as f64 / fs_in as f64;

    let params = SincInterpolationParameters {
        sinc_len: 256,
        f_cutoff: 0.95,
        interpolation: SincInterpolationType::Linear,
        oversampling_factor: 256,
        window: WindowFunction::BlackmanHarris2,
    };
    let mut resampler = SincFixedIn::<f64>::new(
        16000 as f64 / 44100 as f64,
        2.0,
        params,
        1024,
        2,
    ).unwrap();

    let mut input_frames_next = resampler.input_frames_next();
    let resampler_delay = resampler.output_delay();
    let mut outbuffer = vec![vec![0.0f64; resampler.output_frames_max()]; channels];
    let mut indata_slices: Vec<&[f64]> = indata.iter().map(|v| &v[..]).collect();

    let time_for_resample = Instant::now();
    let mut file_out_disk = TokyioFile::create("/Users/dieudonn/Downloads/large-sample-usa-rubato-16khz.raw").await?;

    while indata_slices[0].len() >= input_frames_next {
        let (nbr_in, nbr_out) = resampler
            .process_into_buffer(&indata_slices, &mut outbuffer, None)
            .unwrap();
        for chan in indata_slices.iter_mut() {
            *chan = &chan[nbr_in..];
        }
        append_frames(&mut outdata, &outbuffer, nbr_out);
        let _ = write_frames(&outbuffer, &mut file_out_disk).await?;
        input_frames_next = resampler.input_frames_next();
    }

    if !indata_slices[0].is_empty() {
        let (_nbr_in, nbr_out) = resampler
            .process_partial_into_buffer(Some(&indata_slices), &mut outbuffer, None)
            .unwrap();
        append_frames(&mut outdata, &outbuffer, nbr_out);
        let _ = write_frames(&outbuffer, &mut file_out_disk).await?;
    }

    let nbr_output_frames = (nbr_input_frames as f32 * fs_out as f32 / fs_in as f32) as usize;
    println!("Re-Sample is done.. it took {:?} ... write chunks to disk now...", time_for_resample.elapsed());
    Ok(())
}

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
            wf.push(value);
        }
    }
    wfs
}

async fn write_frames<W: AsyncWriteExt + std::marker::Unpin>(waves: &[Vec<f64>], output: &mut W) -> Result<()> {
    let channels = waves.len();
    let frames_to_skip = 0;
    let frames_to_write = waves[0].len();
    let end = frames_to_skip + frames_to_write;
    for frame in frames_to_skip..end {
        for wave in waves.iter().take(channels) {
            let value64 = wave[frame];
            let bytes = value64.to_le_bytes();
            output.write_all(&bytes).await?;
        }
    }
    Ok(())
}

fn append_frames(buffers: &mut [Vec<f64>], additional: &[Vec<f64>], nbr_frames: usize) {
    buffers
        .iter_mut()
        .zip(additional.iter())
        .for_each(|(b, a)| b.extend_from_slice(&a[..nbr_frames]));
}
