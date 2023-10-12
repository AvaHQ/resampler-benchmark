# resampler-benchmark

This repository is just to have a possibility to remember the benchmark for resampling

## How to try yourself

### Install deps

0. Clone the repo
1. `yarn`
2. Download the audio file **(~60mb)** and rename it `large-sample-usa.wav`
3. Duplicate the `.env.example` file to `.env` and put the right path of the downloaded file

### 🟧 Speex-resampler

It's the library we are using ATM at ava, it's a webassembly using c native code.
It have issue on master branch but is faster on the branch used on ava-backend called `remove-unhandled-rejection-handler` we need to fix that to not re-create the issue. master has one more commit `f9f3320cc8fc61a585489efa5552a5f8ec0c3517` and it pass from 20 seconds on master to 4 secondes of transform inside the branch

To test it uncomment the version you want to test inside `src/speex-resampler`and run `npx ts-node src/speex-resampler.ts`

### 🟧 libsamplerate.js

This library could be tried via the file `src/output-libsamplerate.ts` via `npx ts-node src/libsamplerate.ts` it will output the wav file into `output/-libsamplerate`

## ❌ node-opusenc

This one is not working as expected and have []memory crash from C++ binding](https://github.com/VictorQueiroz/node-opusenc/issues/1). I added it just to remember what was the issue

## ❌ Opusenc binary

1. It's a binary that you can install on mac for ex via `brew install opus-tools`
2. `time opusenc --quiet --comp 0 --raw --raw-rate 44100 --raw /Users/dieudonn/Downloads/large-sample-usa.wav ./output/output-opusenc.opus` (replace full path for sure)

Opus codec [seems to not deal only with a 48Khz rate](https://wiki.xiph.org/OpusFAQ#How_do_I_use_44.1_kHz_or_some_other_sampling_rate_not_directly_supported_by_Opus.3F)

- It have a re-sample tool available but it seems to use [speex under the hood](https://github.com/xiph/opus-tools/blob/master/src/resample.c#L66)

- It cannot resample to 16Khz, it can just take any rate in entrey and normalize to 48Khz so it's a no go

## ✅ Sox binary

This one is really fast and result is good, [it's a C library](https://sourceforge.net/p/sox/code/ci/master/tree/) wrotte in 1991, [some portability in other langage exist](https://fr.wikipedia.org/wiki/SoX#Bibliothèque)

1. Install the binary `brew install sox`
2. Resample the file `time sox /Users/dieudonn/Downloads/large-sample-usa.wav -r 16000 ./output/output-sox.wav`
