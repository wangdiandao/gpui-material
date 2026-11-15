import { defineConfig } from 'astro/config';
import { unified } from '@astrojs/markdown-remark';
import { remarkRepositoryLinks } from './src/markdown.mjs';

const site = process.env.PAGES_ORIGIN || 'https://wangdiandao.github.io';
const base = process.env.PAGES_BASE ?? '/gpui-material/';

export default defineConfig({
  site,
  base: base || '/',
  output: 'static',
  trailingSlash: 'always',
  integrations: [{
    name: 'repository-markdown',
    hooks: {
      'astro:config:setup': ({ config, updateConfig }) => {
        updateConfig({
          markdown: {
            processor: unified({
              remarkPlugins: [[remarkRepositoryLinks, { base: config.base }]],
              shikiConfig: { themes: { light: 'github-light', dark: 'github-dark' } },
            }),
          },
        });
      },
    },
  }],
});
