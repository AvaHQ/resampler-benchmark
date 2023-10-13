import { readFileSync, writeFileSync } from "fs";
import { resolve } from "path";
// ! We have a major GAP of transform between our master branch of our speex-resampler fork (20s->4s)
// ? uncomment the version you want to test
// import SpeexResampler from "speex-resampler"; // original version from the creator
import SpeexResampler from "speex-resampler-ava-version-branch"; //branch version (better)
// import SpeexResampler from "speex-resampler-ava-version-master"; //master version (worst)

(async () => {
  await SpeexResampler.initPromise; // will be resolved once the WASM module has been compiled, before this you cannot call the SpeexResampler processChunk method

  const channels = 2; // minimum is 1, no maximum
  const inRate = 44100; // frequency in Hz for the input chunk
  const outRate = 16000; // frequency in Hz for the target chunk
  const quality = 7; // number from 1 to 10, default to 7, 1 is fast but of bad quality, 10 is slow but best quality
  // you need a new resampler for every audio stream you want to resample
  // it keeps data from previous calls to improve the resampling
  const resampler = new SpeexResampler(
    channels,
    inRate,
    outRate,
    quality // optionnal
  );

  const pcmData = readFileSync(
    process.env.LARGE_WAV_PATH ||
      "/Users/dieudonn/Downloads/large-sample-usa.wav"
  );
  console.log("Start converting sample rate to 16Khz");
  console.time("speex-resampler");
  const res = await resampler.processChunk(pcmData);
  console.timeEnd("speex-resampler");
  console.log("End converting");

  const outputFile = resolve(__dirname, "../output/output-speex-resampler.pcm");
  writeFileSync(outputFile, res);
})();
