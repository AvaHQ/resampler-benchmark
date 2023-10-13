// https://github.com/victorqueiroz/node-opusenc
import opus from "libopus";
import { spawn } from "child_process";
import { resolve } from "path";
import "dotenv/config";

const comments = new opus.opusenc.Comments();
const enc = new opus.opusenc.Encoder();
enc.createFile(
  comments,
  resolve(__dirname, "../output/output-libopus-js.ogg"),
  16000,
  2,
  0
);
const pcm = spawn("sox", [
  process.env.LARGE_WAV_PATH ||
    "/Users/dieudonn/Downloads/large-sample-usa.wav",
  "-t",
  "wav",
  "-",
]);
console.log("Start converting sample rate...");
pcm.stdout.on("data", (chunk) => {
  const samples = Math.floor(chunk.byteLength / Float32Array.BYTES_PER_ELEMENT);
  const buf = new Float32Array(samples);
  buf.set(new Float32Array(chunk.buffer, chunk.byteOffset, samples));
  enc.writeFloat(buf, samples);
});
new Promise<void>((resolve) => {
  pcm.stdout.on("exit", (code) => {
    console.log("Finished converting file");
    enc.drain();
    resolve();
  });
});
