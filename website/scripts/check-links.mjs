import { readFile, readdir, stat } from 'node:fs/promises';
import { join, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parse } from 'parse5';
import { components } from '../component/catalog.mjs';
import { sitePath } from '../src/markdown.mjs';

const dist = fileURLToPath(new URL('../dist/', import.meta.url));
const base = sitePath(process.env.PAGES_BASE ?? '/gpui-material/');
const origin = new URL(process.env.PAGES_ORIGIN || 'https://wangdiandao.github.io').origin;
const pages = new Map();

for (const entry of await readdir(dist, { recursive: true, withFileTypes: true })) {
  if (!entry.isFile() || !entry.name.endsWith('.html')) continue;
  const file = join(entry.parentPath, entry.name);
  const relativePath = file.slice(dist.length).split(sep).join('/');
  const route = base + relativePath.replace(/(^|\/)index\.html$/, '$1');
  const document = parse(await readFile(file, 'utf8'));
  const ids = new Set();
  const links = [];
  const nodes = [document];
  while (nodes.length) {
    const node = nodes.pop();
    for (const { name, value } of node.attrs ?? []) {
      if (name === 'id') ids.add(value);
      if (['href', 'src', 'data-script', 'data-style'].includes(name)) links.push(value);
    }
    nodes.push(...(node.childNodes ?? []));
  }
  pages.set(route, { ids, links });
}

const errors = [];
for (const [route, { links }] of pages) {
  for (const link of links) {
    const target = new URL(link, origin + route);
    if (target.origin !== origin) continue;
    if (!target.pathname.startsWith(base)) {
      errors.push(`${route}: escapes the configured base: ${link}`);
      continue;
    }
    const destination = pages.get(target.pathname);
    if (destination) {
      if (target.hash && !destination.ids.has(decodeURIComponent(target.hash.slice(1)))) {
        errors.push(`${route}: missing anchor: ${link}`);
      }
    } else if (!(await stat(join(dist, decodeURIComponent(target.pathname.slice(base.length)))).catch(() => null))?.isFile()) {
      errors.push(`${route}: missing page or asset: ${link}`);
    }
  }
}

for (const route of ['', 'component/', 'docs/', ...components.map(({ slug }) => `component/${slug}/`)]) {
  for (const locale of ['', 'zh-CN/']) {
    if (!pages.has(base + locale + route)) errors.push(`Missing ${base}${locale}${route}`);
  }
}

const storySource = await readFile(new URL('../../apps/gallery/src/story.rs', import.meta.url), 'utf8');
const galleryStories = new Set([...storySource.matchAll(/=> "([a-z][a-z-]+)",/g)].map(([, story]) => story));
const documentedStories = new Set(['introduction', ...components.flatMap(({ story, relatedStory }) => [story, relatedStory?.id]).filter(Boolean)]);
for (const story of galleryStories.difference(documentedStories)) errors.push(`Gallery story has no guide: ${story}`);
for (const story of documentedStories.difference(galleryStories)) errors.push(`Unknown Gallery story: ${story}`);

if (errors.length) {
  console.error(errors.join('\n'));
  process.exitCode = 1;
} else {
  console.log(`Checked local links in ${pages.size} pages and ${documentedStories.size - 1} Gallery story IDs.`);
}
