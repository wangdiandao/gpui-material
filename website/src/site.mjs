import { components } from '../component/catalog.mjs';
import { docsRoute, sitePath } from './markdown.mjs';
import workspaceManifest from '../../Cargo.toml?raw';

const base = import.meta.env.BASE_URL;
const markdown = import.meta.glob('../../docs/**/*.md', { eager: true });

export const guides = Object.entries(markdown).map(([file, module]) => ({
  route: docsRoute(file.replace(/^\.\.\/\.\.\//, '')),
  module,
}));

export const copy = {
  en: {
    docs: 'Guides', components: 'Components', gallery: 'Gallery', api: 'API reference',
    search: 'Search documentation', skip: 'Skip to content', theme: 'Color theme',
    light: 'Light', dark: 'Dark', system: 'System', language: '简体中文', menu: 'Navigation',
    onThisPage: 'On this page', source: 'Source', usage: 'Usage', tryIt: 'Try in Gallery',
    overview: 'Component catalog', setup: 'Get started', themeGuide: 'Theme and tokens',
    allGuides: 'All guides', feature: 'Cargo feature', platform: 'Native Rust UI for GPUI',
    headline: 'Material 3 Expressive for GPUI',
    introduction: 'Build native desktop applications with Material components for Rust. Browse controls, learn how to wire state and events, and try the interactive examples in the Gallery.',
    catalogIntro: 'Find a control by family. Each guide has a Rust example, notes on state or configuration, and links to the API and Gallery.',
    galleryIntro: 'Run the native Gallery from a source checkout to try variants, keyboard interactions, and light and dark themes.',
    galleryOpen: 'Pass a story ID to open its example directly.',
    sourceOnly: 'Run from the repository root.',
  },
  zh: {
    docs: '指南', components: '组件', gallery: '演示', api: 'API 参考',
    search: '搜索文档', skip: '跳到内容', theme: '配色主题',
    light: '浅色', dark: '深色', system: '跟随系统', language: 'English', menu: '导航',
    onThisPage: '本页目录', source: '源码', usage: '用法', tryIt: '在 Gallery 中体验',
    overview: '组件目录', setup: '快速开始', themeGuide: '主题与 Token',
    allGuides: '全部指南', feature: 'Cargo feature', platform: '面向 GPUI 的 Rust 原生界面',
    headline: '在 GPUI 中使用 Material 3 Expressive',
    introduction: '用 Rust 和 Material 组件构建原生桌面应用。查看控件如何管理状态和处理事件，并在 Gallery 里尝试交互示例。',
    catalogIntro: '按类别浏览控件。每篇指南提供 Rust 示例、状态或配置说明，以及通往 API 与 Gallery 的入口。',
    galleryIntro: '从源码检出运行原生 Gallery，体验组件变体、键盘操作和浅色及深色主题。',
    galleryOpen: '传入演示 ID 即可直接打开对应示例。',
    sourceOnly: '在仓库根目录运行。',
  },
};

export const groups = {
  actions: { en: 'Actions and selection', zh: '操作与选择' },
  content: { en: 'Content and feedback', zh: '内容与反馈' },
  input: { en: 'Input and overlays', zh: '输入与浮层' },
  navigation: { en: 'Navigation and layout', zh: '导航与布局' },
  optional: { en: 'Optional components', zh: '可选组件' },
};

export const api = 'https://docs.rs/gpui-material/latest/gpui_material/';
const workspacePackage = workspaceManifest.split('[workspace.package]')[1]?.split(/^\[/m)[0];
export const packageVersion = workspacePackage?.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
if (!packageVersion) throw new Error('Missing version in [workspace.package]');

export function pagePath(route, lang) {
  return sitePath(base, [lang === 'zh' ? 'zh-CN' : '', route].filter(Boolean).join('/'));
}

export function sidebar(lang, kind) {
  const t = copy[lang];
  if (kind === 'component') {
    return [
      { title: t.components, items: [{ label: t.overview, route: 'component' }] },
      ...Object.entries(groups).map(([group, title]) => ({
        title: title[lang],
        items: components.filter((item) => item.group === group)
          .map((item) => ({ label: item.title, route: `component/${item.slug}` })),
      })),
    ];
  }

  return [{
    title: t.docs,
    items: [
      { label: t.allGuides, route: 'docs' },
      { label: t.setup, route: 'docs/integration' },
      { label: t.themeGuide, route: 'docs/theme/theme' },
      { label: lang === 'zh' ? '控件支持范围' : 'Component coverage', route: 'docs/component' },
      { label: lang === 'zh' ? '架构与依赖' : 'Architecture', route: 'docs/architecture' },
      { label: lang === 'zh' ? '开发与验证' : 'Development', route: 'docs/development' },
      { label: t.gallery, route: 'gallery' },
    ],
  }];
}
