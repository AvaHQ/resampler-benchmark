import ffmpeg from "fluent-ffmpeg";
import { createWriteStream } from "fs";
import { resolve } from "path";
import "dotenv/config";

const outputfile = createWriteStream(
  resolve(__dirname, "../output/output-ffmpeg.wav")
);
console.time("fluentffmpeg");
ffmpeg()
  .input(
    // can send stream buffer too
    process.env.LARGE_WAV_PATH ||
      "/Users/dieudonn/Downloads/large-sample-usa.wav"
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
