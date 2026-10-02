#!/usr/bin/env node
// Optional media build: Node + sharp + ffmpeg, never runtime dependencies.
// cargo run --locked --example habitat -- --frames target/habitat-frames
// node tools/render-habitat.cjs target/habitat-frames assets/readme/habitat-motion.gif
const fs = require('node:fs');
const path = require('node:path');
const os = require('node:os');
const crypto = require('node:crypto');
const { spawnSync } = require('node:child_process');
const sharp = require('sharp');

async function main() {
  const [source = 'target/habitat-frames', output = 'assets/readme/habitat-motion.gif'] = process.argv.slice(2);
  const manifest = fs.readFileSync(path.join(source, 'manifest.json'));
  const frames = JSON.parse(manifest);
  if (frames.length !== 80) throw new Error('expected 80 exported habitat frames');
  const digest = crypto.createHash('sha256').update(manifest);
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), 'codewhale-habitat-'));
  try {
    for (const [index, frame] of frames.entries()) {
      if (frame.file !== `frame-${String(index).padStart(3, '0')}.svg` || frame.width !== 112 || frame.height !== 30 || frame.profile !== 'dark-truecolor' || frame.elapsed_ms !== 6000 + index * 100) throw new Error('unexpected frame manifest');
      const svg = fs.readFileSync(path.join(source, frame.file));
      if (/href\s*=|<script|<foreignObject|<!DOCTYPE|\bon\w+\s*=/i.test(svg.toString())) throw new Error('active or external frame content');
      digest.update(frame.file).update('\0').update(svg);
      await sharp(svg).png().toFile(path.join(temporary, `frame-${String(index).padStart(3, '0')}.png`));
    }
    fs.mkdirSync(path.dirname(output), { recursive: true });
    const result = spawnSync('ffmpeg', ['-v', 'error', '-y', '-framerate', '10', '-i', path.join(temporary, 'frame-%03d.png'), '-filter_complex', '[0:v]split[a][b];[a]palettegen=stats_mode=diff[p];[b][p]paletteuse=dither=none', '-loop', '0', output], { stdio: 'inherit' });
    if (result.error || result.status !== 0) throw result.error || new Error('ffmpeg failed');
    const metadata = await sharp(output, { animated: true }).metadata();
    if (metadata.width !== 1120 || metadata.pageHeight !== 600 || metadata.pages !== 80) throw new Error('GIF does not preserve the exported frame dimensions/count');
    const record = { source_sha256: digest.digest('hex'), gif_sha256: crypto.createHash('sha256').update(fs.readFileSync(output)).digest('hex'), frames: 80, width: 1120, height: 600, frame_ms: 100 };
    fs.writeFileSync(`${output}.json`, JSON.stringify(record, null, 2) + '\n');
    process.stdout.write(`Rendered ${record.frames} actual buffer frames to ${output}\n`);
  } finally { fs.rmSync(temporary, { recursive: true, force: true }); }
}
main().catch(error => { process.stderr.write(error.message + '\n'); process.exitCode = 1; });
