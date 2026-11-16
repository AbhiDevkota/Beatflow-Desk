/*
 * Generates native/ui/icons/*.svg out of the web app's own icon sources, so the
 * native shell draws the very same glyphs:
 *
 *  - web/app/src/components/icons/NavIcons.tsx   (Apple Music sidebar glyphs)
 *  - web/app/src/components/icons/Transport.tsx  (Apple Music player glyphs)
 *  - lucide-static (the same package lucide-react wraps) for the few stock
 *    glyphs (Users, Plus, ListMusic, Settings, window controls, …), with the
 *    stroke widths the components use baked in.
 *
 * Path data is copied verbatim — never retyped — so visual drift is impossible.
 * Every output is a monochrome `fill="#000"`/`stroke="#000"` drawing: Slint's
 * `Image.colorize` tints them at render time (alpha is preserved).
 *
 * Usage:
 *   node native/tools/gen-icons.mjs [lucide-static-icons-dir]
 * (lucide dir: unpack the lucide-static npm tarball; defaults to %TEMP%/package/icons)
 */

import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const repo = resolve(here, '..', '..');
const outDir = resolve(here, '..', 'ui', 'icons');
const navIconsPath = join(repo, 'web', 'app', 'src', 'components', 'icons', 'NavIcons.tsx');
const transportPath = join(repo, 'web', 'app', 'src', 'components', 'icons', 'Transport.tsx');
const lucideDir = resolve(process.argv[2] ?? join(process.env.TEMP ?? '/tmp', 'package', 'icons'));

const kebab = (name) =>
  name.replace(/Icon$/, '').replace(/([a-z0-9])([A-Z])/g, '$1-$2').toLowerCase();

const svgFile = (viewBox, inner) =>
  `<svg xmlns="http://www.w3.org/2000/svg" viewBox="${viewBox}" fill="#000">\n` +
  inner.map((p) => `  <path d="${p.d}"${p.fillOpacity != null ? ` fill-opacity="${p.fillOpacity}"` : ''}${p.transform ? ` transform="${p.transform}"` : ''}${p.opacity != null ? ` opacity="${p.opacity}"` : ''} />`).join('\n') +
  `\n</svg>\n`;

const strokeSvgFile = (viewBox, stroke, inner, extra = '') =>
  `<svg xmlns="http://www.w3.org/2000/svg" viewBox="${viewBox}" fill="none" stroke="#000" stroke-width="${stroke}" stroke-linecap="round" stroke-linejoin="round"${extra}>\n` +
  inner +
  `\n</svg>\n`;

// ---- NavIcons.tsx: export const X = make([{ d, fillOpacity?, transform? }, …]) -------------

const nav = readFileSync(navIconsPath, 'utf8');
for (const m of nav.matchAll(/export const (\w+) = make\(\[([\s\S]*?)\]\);/g)) {
  const [, name, body] = m;
  // Split into per-path object chunks (the d strings contain no braces).
  const chunks = body.split(/(?=\{)/).filter((c) => c.trim().startsWith('{'));
  const paths = chunks.map((chunk) => {
    const d = chunk.match(/d:\s*'([\s\S]*?)'/)?.[1];
    if (!d) return null;
    const fillOpacity = chunk.match(/fillOpacity:\s*([\d.]+)/)?.[1];
    const transform = chunk.match(/transform:\s*'([^']+)'/)?.[1];
    return { d, fillOpacity: fillOpacity === undefined ? null : Number(fillOpacity), transform };
  }).filter(Boolean);
  writeFileSync(join(outDir, `${kebab(name)}.svg`), svgFile('0 0 24 24', paths));
  console.log(`nav    ${kebab(name)}.svg`);
}

// ---- Transport.tsx: export const X = icon(VIEWBOX, ["d", …]) -------------------------------

const transport = readFileSync(transportPath, 'utf8');
const TRANSPORT_VIEWBOX = transport.match(/const TRANSPORT = '([^']+)';/)?.[1];
for (const m of transport.matchAll(/export const (\w+) = icon\((TRANSPORT|'[^']*'),\s*\[([\s\S]*?)\]\);/g)) {
  const [, name, vbRaw, arrBody] = m;
  const viewBox = vbRaw === 'TRANSPORT' ? TRANSPORT_VIEWBOX : vbRaw.replace(/'/g, '');
  const paths = [...arrBody.matchAll(/"([^"]+)"/g)].map((p) => ({ d: p[1] }));
  writeFileSync(join(outDir, `${kebab(name)}.svg`), svgFile(viewBox, paths));
  console.log(`player ${kebab(name)}.svg`);
}

// Previous is Next mirrored (Transport.tsx scales the forward glyph by -1 in x;
// mirrored inside the 2 0 28 28 crop: x' = 32 - x).
const forwardD = transport.match(/export const ForwardIcon = icon\(TRANSPORT, \["([^"]+)"/)?.[1];
if (forwardD) {
  writeFileSync(
    join(outDir, 'backward.svg'),
    svgFile(TRANSPORT_VIEWBOX, [{ d: forwardD, transform: 'translate(32,0) scale(-1,1)' }]),
  );
  console.log('player backward.svg');
}

// VolumeIcon: the speaker body, then three waves, each hidden below its level
// (first at silence, the next two under a third and two thirds) at 0.25 opacity.
const speakerD = transport.match(/const SPEAKER = "([^"]+)";/)?.[1];
const wavesRaw = transport.match(/const WAVES = \[([\s\S]*?)\];/)?.[1];
if (speakerD && wavesRaw) {
  const waves = [...wavesRaw.matchAll(/"([^"]+)"/g)].map((w) => w[1]);
  for (let shown = 0; shown <= 3; shown++) {
    const inner = [
      { d: speakerD },
      ...waves.map((d, i) => ({ d, opacity: i < shown ? 1 : 0.25 })),
    ];
    writeFileSync(join(outDir, `volume-${shown}.svg`), svgFile('0 0 64 64', inner));
    console.log(`player volume-${shown}.svg`);
  }
}

// ---- lucide-static stock glyphs, with the app's stroke widths ------------------------------

// name -> stroke width, matching the strokeWidth props in the React components
// (WindowControls, Sidebar, AccountButton, Lcd, TrackMenu).
const lucideWanted = {
  'users': 2,        // Sidebar "Listen Together" (size 20)
  'plus': 2,         // sidebar "New Playlist" trigger (size 14)
  'list-music': 1.8, // playlist rows (size 17)
  'settings': 1.9,   // sidebar footer gear (size 18)
  'minus': 1.75,     // window minimize (size 14)
  'x': 1.75,         // window close (size 15)
  'square': 1.5,     // window maximize (size 11)
  'maximize-2': 2.25, // LCD hover expand (size 14)
  'ellipsis': 2,     // TrackMenu trigger (size 18)
};
for (const [name, stroke] of Object.entries(lucideWanted)) {
  const p = join(lucideDir, `${name}.svg`);
  if (!existsSync(p)) {
    console.warn(`lucide ${name}.svg not found in ${lucideDir} — skipped`);
    continue;
  }
  const raw = readFileSync(p, 'utf8');
  const inner = raw
    .replace(/^[\s\S]*?<svg[^>]*>/, '')   // drop the svg open tag + license comment
    .replace(/<\/svg>\s*$/, '')
    .replace(/<!--[\s\S]*?-->/g, '')
    .trim();
  const viewBox = raw.match(/viewBox="([^"]+)"/)?.[1] ?? '0 0 24 24';
  writeFileSync(join(outDir, `${name}.svg`), strokeSvgFile(viewBox, stroke, inner));
  console.log(`lucide ${name}.svg (stroke ${stroke})`);
}

// ---- Hand-copied glyphs from the app's own source ------------------------------------------

// WindowControls.tsx: the restore glyph (two overlapping squares), 12×12, stroke 1.2.
writeFileSync(
  join(outDir, 'restore.svg'),
  strokeSvgFile('0 0 12 12', 1.2, `<rect x="3.5" y="1.5" width="7" height="7" rx="0.5" />\n  <path d="M1.5 3.5v7h7" />`),
);
console.log('manual restore.svg');

// AccountButton.tsx: the official Google "G", its four brand colours preserved
// (rendered without colorize in Slint).
const googleG = [
  ['EA4335', 'M24,9.5c3.54,0 6.71,1.22 9.21,3.6l6.85,-6.85C35.9,2.38 30.47,0 24,0 14.62,0 6.51,5.38 2.56,13.22l7.98,6.19C12.43,13.72 17.74,9.5 24,9.5z'],
  ['4285F4', 'M46.98,24.55c0,-1.57 -0.15,-3.09 -0.38,-4.55H24v9.02h12.94c-0.58,2.96 -2.26,5.48 -4.78,7.18l7.73,6c4.51,-4.18 7.09,-10.36 7.09,-17.65z'],
  ['FBBC05', 'M10.53,28.59c-0.48,-1.45 -0.76,-2.99 -0.76,-4.59s0.27,-3.14 0.76,-4.59l-7.98,-6.19C0.92,16.46 0,20.12 0,24c0,3.88 0.92,7.54 2.56,10.78l7.97,-6.19z'],
  ['34A853', 'M24,48c6.48,0 11.93,-2.13 15.89,-5.81l-7.73,-6c-2.15,1.45 -4.92,2.3 -8.16,2.3 -6.26,0 -11.57,-4.22 -13.47,-9.91l-7.98,6.19C6.51,42.62 14.62,48 24,48z'],
];
writeFileSync(
  join(outDir, 'google-g.svg'),
  `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48">\n` +
  googleG.map(([fill, d]) => `  <path fill="#${fill}" d="${d}" />`).join('\n') +
  `\n</svg>\n`,
);
console.log('manual google-g.svg');

// LogoMark.tsx: web/app/src/assets/logo-mark.svg, recoloured to black so
// colorize can tint it.
const logo = readFileSync(join(repo, 'web', 'app', 'src', 'assets', 'logo-mark.svg'), 'utf8');
const logoPaths = [...logo.matchAll(/<path([^>]*)\/>/g)].map((p) => {
  const attrs = p[1];
  const d = attrs.match(/d="([^"]+)"/)?.[1];
  const rule = attrs.match(/fill-rule="([^"]+)"/)?.[1];
  return { d, rule };
}).filter((p) => p.d);
writeFileSync(
  join(outDir, 'logo-mark.svg'),
  `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1241 1241" fill="#000">\n` +
  logoPaths.map((p) => `  <path fill-rule="${p.rule ?? 'evenodd'}" d="${p.d}" />`).join('\n') +
  `\n</svg>\n`,
);
console.log('manual logo-mark.svg');

console.log('done.');
