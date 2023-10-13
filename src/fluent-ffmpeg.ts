import ffmpeg from "fluent-ffmpeg";
import { createWriteStream, readFileSync } from "fs";
import { writeFile } from "fs/promises";
import { resolve } from "path";
import "dotenv/config";
import { Stream } from "stream";

const outputfile = createWriteStream(
  resolve(__dirname, "../output/output-ffmpeg.wav")
);
const inputFile =
  process.env.LARGE_WAV_PATH ||
  "/Users/dieudonn/Downloads/large-sample-usa.wav";
console.time("fluentffmpeg");
ffmpeg()
  .input(
    // can send stream buffer too
    inputFile
  )
  // .inputFormat("s16le") // Format audio d'entrée (ex. PCM 16-bit little-endian) should be set for stream
  .audioChannels(2)
  .audioFrequency(16000) //re-sample
  .on("end", function () {
    console.timeEnd("fluentffmpeg");
    console.log("Finished");
  })
  .on("error", function (err) {
    console.error("Erreur :", err);
  })
  .format("wav")
  .on("data", console.log)
  .writeToStream(outputfile);

// Async 70 big files to transform at the same time, to check ffmpeg-fluent scalabity

const wavInputBuffer = readFileSync(inputFile);
const listOfBufferFilesToTransform = Array(70)
  .fill(null)
  .map(async () => {
    const randomNb = Math.random();
    const outputNameWithtimeStamp = resolve(
      __dirname,
      `../output/output-ffmpeg-${randomNb}.wav`
    );
    const outputBuffer = await convertToBuffer(wavInputBuffer).then((buffer) =>
      writeFile(outputNameWithtimeStamp, buffer)
    );
    return outputBuffer;
  });

// here we are running 100 time the transform from the buffer copies
(async () => {
  console.time("multipleAsyncFluent");
  const buffers = await Promise.all(listOfBufferFilesToTransform);
  console.timeEnd("multipleAsyncFluent");
})();

async function convertToBuffer(inputBuffer: Buffer): Promise<Buffer> {
  const bufferStreamOutput = new Stream.PassThrough();
  const buffers = [];
  return new Promise((resolve, reject) => {
    bufferStreamOutput.on("data", function (buf) {
      buffers.push(buf);
    });
    bufferStreamOutput.on("end", function () {
      const outputBuffer = Buffer.concat(buffers);
      resolve(outputBuffer);
    });
    const command = ffmpeg({ source: Stream.Readable.from(inputBuffer) })
      // .inputFormat('s16le') // Format audio d'entrée (ex. PCM 16-bit little-endian)
      .audioChannels(2) // Nombre de canaux audio
      .audioFrequency(chooseRandomSampleRate()) // Nouveau taux d'échantillonnage (16000)
      .format("wav")
      // .toFormat(outputFormat); // Format de sortie souhaité
      .writeToStream(bufferStreamOutput);

    command.on("end", () => {
      bufferStreamOutput.end();
    });

    command.on("error", (err) => {
      reject(err);
    });
  });
}

function chooseRandomSampleRate() {
  const sampleRates = [16000, 32000, 44100, 48000];

  const randomIndex = Math.floor(Math.random() * 4);

  return sampleRates[randomIndex];
}
