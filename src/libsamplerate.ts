import { create, ConverterType } from "@alexanderolsen/libsamplerate-js";
import { readFileSync, writeFileSync } from "fs";
import { resolve } from "path";
import { WaveFile } from "wavefile";
import "dotenv/config";

const inFile = process.env.LARGE_WAV_PATH;
const outFile = resolve(__dirname, "../output/output-libsamplerate.wav");

const data = readFileSync(inFile, { encoding: "base64" });
const wav = new WaveFile();
wav.fromBase64(data);

const nChannels = wav.fmt["numChannels"] as number;
const inputSampleRate = wav.fmt["sampleRate"] as number;

let converterType = ConverterType.SRC_SINC_BEST_QUALITY;
let outputSampleRate = 16000;

create(nChannels, inputSampleRate, outputSampleRate, {
  converterType: converterType, // default SRC_SINC_FASTEST. see API for more
}).then((src) => {
  console.log("Start converting sample rate");
  console.time("duration");
  let samples = wav.getSamples(true, Int16Array);
  let float32 = new Float32Array(samples);
  for (var i = 0; i < samples.length; i++) {
    float32[i] = samples[i] / 32767;
  }

  const newSamples = src.simple(float32);
  for (let i = 0; i < newSamples.length; i++) {
    newSamples[i] = newSamples[i] * 32767;
  }
  console.timeEnd("duration");
  console.log("End converting sample rate");

  const newWav = new WaveFile();
  newWav.fromScratch(nChannels, outputSampleRate, "16", newSamples);
  writeFileSync(outFile, newWav.toBuffer());
  src.destroy(); // clean up
});
