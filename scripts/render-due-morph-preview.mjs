import { mkdirSync, copyFileSync, writeFileSync } from 'node:fs';
import { resolve, join } from 'node:path';
import { execFileSync } from 'node:child_process';

// Optional documentation export; ffmpeg is not an application dependency.
for (const browser of ['chromium', 'webkit']) {
  const source = resolve('output/playwright/due-liquid-morph', browser);
  const destination = resolve('docs/design/due-liquid-morph', browser);
  mkdirSync(destination, { recursive: true });
  for (const theme of ['light', 'dark']) {
    const times = [0, 50, 100, 150, 200, 250, 317, 400];
    const frame = time => join(source, `${theme}-${String(time).padStart(3, '0')}.png`);
    const lines = times.flatMap((time, index) => [
      `file '${frame(time).replaceAll("'", "'\\''")}'`,
      `duration ${index + 1 < times.length ? (times[index + 1] - time) / 1000 : 1.1}`,
    ]);
    lines.push(`file '${frame(400).replaceAll("'", "'\\''")}'`);
    const playlist = join(source, `${theme}.ffconcat`);
    writeFileSync(playlist, `${lines.join('\n')}\n`);
    execFileSync('ffmpeg', ['-y', '-loglevel', 'error', '-f', 'concat', '-safe', '0', '-i', playlist,
      '-vf', 'fps=25,split[s0][s1];[s0]palettegen=stats_mode=diff[p];[s1][p]paletteuse=dither=bayer',
      '-loop', '0', join(destination, `${theme}-opening.gif`)]);
    copyFileSync(frame(400), join(destination, `${theme}-rest.png`));
    copyFileSync(join(source, `${theme}-violet-rest.png`), join(destination, `${theme}-violet-rest.png`));
  }
}
