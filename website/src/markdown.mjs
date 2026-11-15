import { posix, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { visit } from 'unist-util-visit';

const repositoryRoot = fileURLToPath(new URL('../../', import.meta.url));
export const repository = 'https://github.com/wangdiandao/gpui-material';

export function docsRoute(source) {
  const zh = source.endsWith('.zh.md');
  const slug = source.slice(0, zh ? -6 : -3).replace(/\/index$/, '');
  return `${zh ? 'zh-CN/' : ''}${slug}`;
}

export function sitePath(base, route = '') {
  return `/${[base, route].map((part) => part.replace(/^\/+|\/+$/g, '')).filter(Boolean).join('/')}`.replace(/\/?$/, '/');
}

export function remarkRepositoryLinks({ base }) {
  return (tree, file) => {
    if (!file.path) return;
    const source = relative(repositoryRoot, resolve(file.path)).split(sep).join('/');

    visit(tree, ['link', 'definition', 'image'], (node) => {
      if (/^(?:[a-z][a-z\d+.-]*:|\/\/|#)/i.test(node.url)) return;
      const [, location, suffix] = node.url.match(/^([^?#]*)(.*)$/);
      if (!location) return;

      const target = location.startsWith('/')
        ? posix.normalize(location.slice(1))
        : posix.join(posix.dirname(source), location);
      if (target.startsWith('docs/') && target.endsWith('.md')) {
        node.url = sitePath(base, docsRoute(target)) + suffix;
      } else {
        const mode = node.type === 'image' ? 'raw' : target.endsWith('/') ? 'tree' : 'blob';
        node.url = `${repository}/${mode}/main/${target}${suffix}`;
      }
    });
  };
}
