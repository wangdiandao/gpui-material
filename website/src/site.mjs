import { components } from '../component/catalog.mjs';
import { docsRoute, sitePath } from './markdown.mjs';
import workspaceManifest from '../../Cargo.toml?raw';

const base = import.meta.env.BASE_URL;
const markdown = import.meta.glob('../../docs/**/*.md', { eager: true });

export const projectName = 'GPUI Material';

export function pageTitle(title = '') {
  return title && !title.includes(projectName) ? `${title} · ${projectName}` : title || projectName;
}

export const guides = Object.entries(markdown).map(([file, module]) => ({
  route: docsRoute(file.replace(/^\.\.\/\.\.\//, '')),
  module,
}));

export const copy = {
  en: {
    docs: 'Docs', components: 'Components', api: 'API reference',
    search: 'Search documentation', skip: 'Skip to content', theme: 'Color theme',
    light: 'Light', dark: 'Dark', system: 'System', language: '简体中文', menu: 'Navigation',
    themeNav: 'Theme', resources: 'Resources', releases: 'Releases', contribute: 'Contribute', close: 'Close',
    capabilities: 'What you can build', architecture: 'How it fits together', preview: 'Browser preview', componentPreview: 'Component preview',
    previewNote: 'This example runs the Rust Gallery with WebAssembly.',
    frontendLicenses: 'Frontend licenses',
    onThisPage: 'On this page', source: 'Source', imports: 'Import', usage: 'Usage',
    examples: 'Examples', references: 'References', copyCode: 'Copy', copiedCode: 'Copied',
    overview: 'Component catalog', setup: 'Get started', themeGuide: 'Theme and tokens',
    allGuides: 'All guides', feature: 'Cargo feature', platform: 'Native Rust UI for GPUI',
    headline: 'Material 3 Expressive for GPUI',
    introduction: `${projectName} provides Material components, semantic themes, and adaptive layouts for native Rust desktop applications built with GPUI.`,
    catalogIntro: 'Browse components by purpose or look up a Rust name within each group.',
    catalogFeatures: 'Enable the listed Cargo feature on gpui-material to use an optional component.',
  },
  zh: {
    docs: '文档', components: '组件', api: 'API 参考',
    search: '搜索文档', skip: '跳到内容', theme: '配色主题',
    light: '浅色', dark: '深色', system: '跟随系统', language: 'English', menu: '导航',
    themeNav: '主题', resources: '资源', releases: '版本发布', contribute: '参与贡献', close: '关闭',
    capabilities: '组件与布局', architecture: '项目分层', preview: '浏览器预览', componentPreview: '组件预览',
    previewNote: '此示例通过 WebAssembly 运行 Rust Gallery。',
    frontendLicenses: '前端许可证',
    onThisPage: '本页目录', source: '源码', imports: '导入', usage: '用法',
    examples: '示例', references: '参考', copyCode: '复制', copiedCode: '已复制',
    overview: '组件目录', setup: '快速开始', themeGuide: '主题与 Token',
    allGuides: '全部指南', feature: 'Cargo feature', platform: '面向 GPUI 的 Rust 原生界面',
    headline: 'GPUI 的 Material 3 Expressive 控件',
    introduction: `${projectName} 为基于 GPUI 的 Rust 原生桌面应用提供 Material 控件、语义主题和自适应布局。`,
    catalogIntro: '按用途浏览控件，或在各组内按 Rust 名称查找。',
    catalogFeatures: '使用可选控件时，在 gpui-material 依赖中启用该控件所列的 Cargo feature。',
  },
};

export const groups = {
  basic: { en: 'Basic components', zh: '基础组件' },
  form: { en: 'Form components', zh: '表单组件' },
  layout: { en: 'Layout components', zh: '布局组件' },
  advanced: { en: 'Advanced components', zh: '高级组件' },
};

export const componentIndex = components.toSorted((left, right) => left.title.localeCompare(right.title, 'en'));

export const capabilities = [
  {
    title: { en: 'Actions and forms', zh: '操作与表单' },
    text: { en: 'Compose buttons, selection controls, and text fields with application-owned values.', zh: '使用按钮、选择控件和文本字段组合表单，由应用持有业务值。' },
    components: ['button', 'text-field', 'exposed-dropdown'],
  },
  {
    title: { en: 'Feedback and overlays', zh: '反馈与浮层' },
    text: { en: 'Ask for confirmation, open supporting content, and show outcomes in a snackbar.', zh: '用对话框确认操作，用面板补充内容，用消息条反馈结果。' },
    components: ['dialog', 'sheet', 'snackbar'],
  },
  {
    title: { en: 'Adaptive navigation', zh: '自适应导航' },
    text: { en: 'Switch navigation with the available width and present list and detail views side by side when space allows.', zh: '根据可用宽度改用不同导航形式，在空间充足时并排显示列表和详情。' },
    components: ['adaptive-layout', 'list-detail', 'supporting-pane'],
  },
];

export const layers = [
  {
    name: 'gpui_material',
    title: { en: 'Components', zh: '控件' },
    text: { en: 'Import Material controls and optional component families from the public facade.', zh: '从公开门面导入 Material 控件，通过 Cargo feature 启用可选组件。' },
    route: 'component',
  },
  {
    name: 'gpui_material::theme',
    title: { en: 'Theme', zh: '主题' },
    text: { en: 'Use semantic color, typography, shape, and motion tokens from a GPUI-independent crate.', zh: '通过独立于 GPUI 的主题 crate 使用语义颜色、排版、形状和动效 Token。' },
    route: 'docs/theme/theme',
  },
  {
    name: 'gpui_material::foundation',
    title: { en: 'Foundation', zh: '基础设施' },
    text: { en: 'Initialize Material windows and connect the theme to GPUI and Base services.', zh: '初始化 Material 窗口，把主题接入 GPUI 和 Base 的交互服务。' },
    route: 'docs/architecture',
  },
];

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
        items: componentIndex.filter((item) => item.group === group)
          .map((item) => ({ label: item.title, route: `component/${item.slug}` })),
      })),
    ];
  }

  return [
    {
      title: t.docs,
      items: [
        { label: t.allGuides, route: 'docs' },
        { label: t.setup, route: 'docs/integration' },
        { label: lang === 'zh' ? '实战手册' : 'Cookbook', route: 'docs/cookbook' },
        { label: lang === 'zh' ? '控件支持范围' : 'Component coverage', route: 'docs/component' },
      ],
    },
    {
      title: t.themeGuide,
      items: [
        { label: lang === 'zh' ? '主题系统' : 'Theme system', route: 'docs/theme/theme' },
        { label: lang === 'zh' ? 'Token 索引' : 'Token index', route: 'docs/theme/token-index' },
      ],
    },
    {
      title: t.architecture,
      items: [
        { label: lang === 'zh' ? '架构与依赖' : 'Architecture and dependencies', route: 'docs/architecture' },
        { label: lang === 'zh' ? '设计依据' : 'Design sources', route: 'docs/design-sources' },
        { label: lang === 'zh' ? '开发指南' : 'Development', route: 'docs/development' },
        { label: t.releases, route: 'docs/release' },
      ],
    },
  ];
}
