import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs'
import { dirname, isAbsolute, join, relative, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { defineConfig, type DefaultTheme } from 'vitepress'
import type MarkdownIt from 'markdown-it'
import carveGrammar from './carve.tmLanguage.json' with { type: 'json' }

const docsDir = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const repoRoot = resolve(docsDir, '..')
const repo = 'https://github.com/void1z2/carve'

const inside = (dir: string, p: string) => {
  const r = relative(dir, p)
  return r !== '' && !r.startsWith('..') && !isAbsolute(r)
}

/**
 * Relative links that leave docs/ (like `../README.md` or `../examples/`) point
 * to the file on GitHub. Links in the included CHANGELOG.md are written from
 * the repository root (`docs/admin.md`), so they are resolved from there.
 */
function repoLinks(md: MarkdownIt) {
  md.core.ruler.push('carve_repo_links', (state) => {
    const rel: string | undefined = state.env?.relativePath
    if (!rel) return
    const fileDir = dirname(join(docsDir, rel))
    for (const block of state.tokens) {
      for (const t of block.children ?? []) {
        if (t.type !== 'link_open') continue
        const href = t.attrGet('href')
        if (!href || /^([a-z]+:|#|\/)/i.test(href)) continue
        const [path, hash] = href.split('#', 2)
        const suffix = hash ? `#${hash}` : ''
        let target = resolve(fileDir, decodeURI(path))
        if (!existsSync(target) && !existsSync(`${target}.md`)) {
          const fromRoot = resolve(repoRoot, decodeURI(path))
          if (!existsSync(fromRoot)) continue
          target = fromRoot
        }
        if (inside(docsDir, target)) {
          if (target !== resolve(fileDir, decodeURI(path))) {
            t.attrSet('href', relative(fileDir, target).replace(/\\/g, '/') + suffix)
          }
          continue
        }
        if (!existsSync(target) || !inside(repoRoot, target)) continue
        const kind = statSync(target).isDirectory() ? 'tree' : 'blob'
        const repoPath = relative(repoRoot, target).replace(/\\/g, '/')
        t.attrSet('href', `${repo}/${kind}/master/${repoPath}${suffix}`)
      }
    }
  })
}

/** First `# Heading` of a Markdown file, or its file name. */
function titleOf(file: string): string {
  const text = readFileSync(file, 'utf8')
  const m = text.match(/^#\s+(.+)$/m)
  return m ? m[1].replace(/`/g, '').trim() : file
}

/** Pages under docs/syntax that have a fixed place in the Language group. */
const fixedSyntax = ['triggers', 'effects', 'conditions', 'expressions', 'loops', 'functions', 'types']

/** Every other docs/syntax/*.md page, found at build time so new ones show up. */
function moreSyntax(): DefaultTheme.SidebarItem[] {
  return readdirSync(join(docsDir, 'syntax'))
    .filter((f) => f.endsWith('.md') && !fixedSyntax.includes(f.slice(0, -3)))
    .map((f) => ({ text: titleOf(join(docsDir, 'syntax', f)), link: `/syntax/${f.slice(0, -3)}` }))
    .sort((a, b) => a.text.localeCompare(b.text))
}

/** A sidebar item only when its page exists. */
function optional(page: string, text: string): DefaultTheme.SidebarItem[] {
  return existsSync(join(docsDir, `${page}.md`)) ? [{ text, link: `/${page}` }] : []
}

const integrations = [...optional('skbee', 'SkBee'), ...optional('placeholders', 'Placeholders')]

const sidebar: DefaultTheme.SidebarItem[] = [
  {
    text: 'Guide',
    items: [
      { text: 'Getting started', link: '/getting-started' },
      { text: 'Install', link: '/getting-started#install' },
      { text: 'First script', link: '/getting-started#your-first-script' },
      { text: 'Hosted servers', link: '/getting-started#hosted-servers' },
      { text: 'Troubleshooting', link: '/getting-started#troubleshooting' },
      { text: 'Reading errors', link: '/errors' },
    ],
  },
  {
    text: 'Language',
    items: [
      { text: 'Triggers', link: '/syntax/triggers' },
      { text: 'Commands', link: '/commands' },
      { text: 'Effects', link: '/syntax/effects' },
      { text: 'Conditions', link: '/syntax/conditions' },
      { text: 'Expressions', link: '/syntax/expressions' },
      { text: 'Loops', link: '/syntax/loops' },
      { text: 'Functions', link: '/syntax/functions' },
      { text: 'Types', link: '/syntax/types' },
      { text: 'Options', link: '/syntax/functions#options' },
      { text: 'Variables', link: '/variables' },
      { text: 'More syntax', collapsed: false, items: moreSyntax() },
    ],
  },
  {
    text: 'Admin',
    items: [
      { text: 'The /carve command', link: '/admin' },
      { text: 'Permissions', link: '/admin#permissions' },
      { text: 'Limits', link: '/limits' },
      { text: 'Performance', link: '/performance' },
    ],
  },
  {
    text: 'Reference',
    items: [
      { text: 'Search all syntax', link: '/reference/' },
      { text: 'Events', link: '/reference/events' },
      { text: 'Effects', link: '/reference/effects' },
      { text: 'Conditions', link: '/reference/conditions' },
      { text: 'Expressions', link: '/reference/expressions' },
      { text: 'Types', link: '/reference/types' },
      { text: 'Functions', link: '/reference/functions' },
    ],
  },
  { text: 'Skript parity', items: [{ text: 'Skript parity', link: '/skript-parity' }] },
  ...(integrations.length ? [{ text: 'Integrations', items: integrations }] : []),
  {
    text: 'Project',
    items: [
      { text: 'Contributing', link: '/contributing' },
      { text: 'Architecture', link: '/architecture' },
      { text: 'In-game testing', link: '/ingame-testing' },
      { text: 'Changelog', link: '/changelog' },
    ],
  },
]

export default defineConfig({
  title: 'Carve',
  description: 'Simple, readable scripts for the Pumpkin Minecraft server',
  base: '/carve/',
  srcDir: '.',
  cleanUrls: true,
  lastUpdated: true,
  appearance: 'dark',
  head: [
    ['link', { rel: 'icon', href: '/carve/favicon.ico' }],
    ['meta', { name: 'theme-color', content: '#f28c28' }],
    ['meta', { property: 'og:title', content: 'Carve' }],
    ['meta', { property: 'og:description', content: 'Simple, readable scripts for the Pumpkin Minecraft server' }],
  ],
  markdown: {
    languages: [carveGrammar as any],
    config: repoLinks,
  },
  themeConfig: {
    logo: { src: '/carve-icon.png', alt: 'Carve' },
    nav: [
      { text: 'Guide', link: '/getting-started' },
      { text: 'Syntax', link: '/syntax/triggers' },
      { text: 'Reference', link: '/reference/' },
      { text: 'Changelog', link: '/changelog' },
    ],
    sidebar,
    socialLinks: [{ icon: 'github', link: repo }],
    search: { provider: 'local' },
    editLink: {
      pattern: `${repo}/edit/master/docs/:path`,
      text: 'Edit this page on GitHub',
    },
    outline: { level: [2, 3], label: 'On this page' },
    lastUpdated: { text: 'Last updated' },
    footer: {
      message: 'Released under the MIT or Apache-2.0 license.',
      copyright: 'Carve is not affiliated with Mojang or the Pumpkin project.',
    },
  },
})
