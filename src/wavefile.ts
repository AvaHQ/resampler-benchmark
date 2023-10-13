import { readFileSync, writeFileSync } from "fs";
import { WaveFile } from "wavefile";
import "dotenv/config";
import { resolve } from "path";

const outputfile = resolve(__dirname, "../output/output-wavefile.wav");
const audio = readFileSync("/Users/dieudonn/Downloads/large-sample-usa.wav");
console.time("wavefile");
const wav = new WaveFile();

wav.fromBuffer(audio);
wav.toBitDepth("16", true);
wav.toSampleRate(16000);
console.timeEnd("wavefile");

writeFileSync(outputfile, wav.toBuffer());
