# resampler-benchmark

This repository is just to have a possibility to remember the benchmark for resampling

## How to try yourself

### Install deps

0. Clone the repo
1. `yarn`
2. Download the audio file **(~60mb)** and rename it `large-sample-usa.wav`
3. Duplicate the `.env.example` file to `.env` and put the right path of the downloaded file

### libsamplerate.js

This library could be tried via the file `src/output-libsamplerate.ts` via `npx ts-node src/libsamplerate.ts` it will output the wav file into `output/-libsamplerate`
