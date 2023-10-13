# resampler-benchmark

This repository is just to have a possibility to remember the benchmark for resampling

## How to try yourself

### Install deps

0. Clone the repo
1. `yarn`
2. Download the audio file **(~60mb)** and rename it `large-sample-usa.wav`
3. Duplicate the `.env.example` file to `.env` and put the right path of the downloaded file

### 🟧 Speex-resampler (4s)

It's the library we are using ATM at ava, it's a webassembly using c native code.
It have issue on master branch but is faster on the branch used on ava-backend called `remove-unhandled-rejection-handler` we need to fix that to not re-create the issue. master has one more commit `f9f3320cc8fc61a585489efa5552a5f8ec0c3517` and it pass from 20 seconds on master to 4 secondes of transform inside the branch

To test it uncomment the version you want to test inside `src/speex-resampler`and run `npx ts-node src/speex-resampler.ts`
![Capture d’écran 2023-10-12 à 11 30 50](https://github.com/AvaHQ/resampler-benchmark/assets/7901366/d926a8ea-8938-4b03-b53e-3c83d0e193ae)



### 🟧 libsamplerate.js (8s)

This library could be tried via the file `src/output-libsamplerate.ts` via `npx ts-node src/libsamplerate.ts` it will output the wav file into `output/-libsamplerate`
![libsamplerate](https://github.com/AvaHQ/resampler-benchmark/assets/7901366/5db8c7aa-e25e-4385-8104-4553b3de2f5c)

## ❌ node-opusenc

This one is not working as expected and have []memory crash from C++ binding](https://github.com/VictorQueiroz/node-opusenc/issues/1). I added it just to remember what was the issue

## ❌ Opusenc binary

1. It's a binary that you can install on mac for ex via `brew install opus-tools`
2. `time opusenc --quiet --comp 0 --raw --raw-rate 44100 --raw /Users/dieudonn/Downloads/large-sample-usa.wav ./output/output-opusenc.opus` (replace full path for sure)

Opus codec [seems to not deal only with a 48Khz rate](https://wiki.xiph.org/OpusFAQ#How_do_I_use_44.1_kHz_or_some_other_sampling_rate_not_directly_supported_by_Opus.3F)

- It have a re-sample tool available but it seems to use [speex under the hood](https://github.com/xiph/opus-tools/blob/master/src/resample.c#L66)

- It cannot resample to 16Khz, it can just take any rate in entrey and normalize to 48Khz so it's a no go

## ✅ Sox binary (0.2s)

This one is really fast and result is good, [it's a C library](https://sourceforge.net/p/sox/code/ci/master/tree/) wrotte in 1991, [some portability in other langage exist](https://fr.wikipedia.org/wiki/SoX#Bibliothèque)

1. Install the binary `brew install sox`
2. Resample the file `time sox /Users/dieudonn/Downloads/large-sample-usa.wav -r 16000 ./output/output-sox.wav`
![sox-binary](https://github.com/AvaHQ/resampler-benchmark/assets/7901366/b927ce04-3e24-4a85-8559-22d882be1919)

## 🟧 Speex binary (2s)

It's the binary speex itself , you will need to send it pcm file, so convert it first !
1.  `brew install speex` 
to try if `time speexenc --vbr --quality 5 --stereo --rate 16000 /Users/dieudonn/Downloads/large-sample-usa.pcm  output/output-speex.wav`

![spee-binary](https://github.com/AvaHQ/resampler-benchmark/assets/7901366/164b2a29-c27a-4b56-aa1f-f3a40b7e8d55)


## ✅ Rubato Rust library (0.2s-0.4s)

[This library](https://github.com/HEnquist/rubato) can be used in a Rust program directly, there is [4/5](https://github.com/HEnquist/rubato/tree/master/examples) different options to tests that have better result of better speed.
Two way of testing them:

1. In this repository you can setup the .env file correctly
2. Run `cargo run --release`
3. The time took to convert if fast, but the time to write the file can be slow :S.

Or you can test it in the github rubato folder
with this [kind of command](https://github.com/HEnquist/rubato/blob/master/examples/process_f64.rs#L24C5-L24C98) `argo run --release --example process_f64 SincFixedIn sine_f64_2ch.raw test.raw 44100 192000 2`
![rubato](https://github.com/AvaHQ/resampler-benchmark/assets/7901366/f63872f0-3f3b-43e2-a9c3-a1b61b89c80b)

## 🟧 Wavefile (~11s)

Based on the [full JS wavefile package](https://www.npmjs.com/package/wavefile#change-the-sample-rate) , easy to use but very slow to convert the file

1. `yarn` to install deps
2. Setup the `.env` file correctly
3. run `npx ts-node src/wavefile.ts`

![Capture d’écran 2023-10-13 à 10 56 18](https://github.com/AvaHQ/resampler-benchmark/assets/7901366/35a50e81-9248-4cd6-bda0-620a7a5e088b)


## ✅ fluent-ffmpeg (0.3s)

Based on the [ffmpeg-package for node](https://www.npmjs.com/package/fluent-ffmpeg) , easy to use with buffer and stream and fast! It need ffmpeg to be installed

1. `yarn` to install deps
2. Setup the `.env` file correctly
3. run `npx ts-node src/fluent-ffmpeg.ts`
