import { defineConfig } from 'vitepress'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  title: 'koban',
  description:
    'A small, scriptable Rust CLI and client library for Invoice Ninja — good for humans at a terminal, predictable for agents with stable JSON, explicit errors, and shell completions.',
  base: '/koban/',

  vite: {
    plugins: [tailwindcss()],
    server: {
      allowedHosts: true,
    },
  },

  head: [
    [
      'link',
      { rel: 'icon', href: '/koban/favicon.svg', type: 'image/svg+xml' },
    ],
    ['meta', { name: 'theme-color', content: '#D4AF37' }],
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:title', content: 'koban' }],
    [
      'meta',
      {
        property: 'og:description',
        content:
          'Invoice Ninja from the terminal — CLI and Rust client library.',
      },
    ],
  ],

  lastUpdated: true,

  markdown: {
    theme: {
      light: 'catppuccin-latte',
      dark: 'catppuccin-mocha',
    },
  },

  sitemap: {
    hostname: 'https://jamesbrink.github.io/koban/',
  },

  themeConfig: {
    logo: '/favicon.svg',
    siteTitle: 'koban',

    nav: [
      { text: 'Guide', link: '/guide/' },
      { text: 'Commands', link: '/commands/' },
      { text: 'Library', link: '/library/' },
      { text: 'Reference', link: '/reference/resource-families' },
      {
        text: 'Links',
        items: [
          {
            text: 'Releases',
            link: 'https://github.com/jamesbrink/koban/releases',
          },
          {
            text: 'koban on crates.io',
            link: 'https://crates.io/crates/koban',
          },
          {
            text: 'koban-cli on crates.io',
            link: 'https://crates.io/crates/koban-cli',
          },
          { text: 'docs.rs', link: 'https://docs.rs/koban' },
        ],
      },
    ],

    sidebar: {
      '/guide/': [
        {
          text: 'Getting Started',
          items: [
            { text: 'What is koban?', link: '/guide/' },
            { text: 'Installation', link: '/guide/installation' },
            { text: 'Quickstart', link: '/guide/quickstart' },
            { text: 'Configuration', link: '/guide/configuration' },
          ],
        },
        {
          text: 'Using koban',
          items: [
            { text: 'Output: tables & JSON', link: '/guide/output' },
            { text: 'Shell completions', link: '/guide/completions' },
            { text: 'Updating', link: '/guide/updating' },
            { text: 'Safety & guardrails', link: '/guide/safety' },
          ],
        },
      ],
      '/commands/': [
        {
          text: 'Commands',
          items: [
            { text: 'Overview', link: '/commands/' },
            { text: 'Resource commands', link: '/commands/resources' },
            { text: 'Invoices', link: '/commands/invoices' },
            { text: 'Endpoint runners', link: '/commands/endpoints' },
          ],
        },
      ],
      '/library/': [
        {
          text: 'Library',
          items: [
            { text: 'Using koban as a library', link: '/library/' },
            { text: 'Typed models', link: '/library/models' },
            { text: 'Resource accessors', link: '/library/resources' },
            { text: 'Errors & features', link: '/library/errors' },
          ],
        },
      ],
      '/reference/': [
        {
          text: 'Reference',
          items: [
            { text: 'Resource families', link: '/reference/resource-families' },
            { text: 'Environment', link: '/reference/environment' },
          ],
        },
      ],
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/jamesbrink/koban' },
    ],

    search: {
      provider: 'local',
    },

    footer: {
      message: 'Released under the MIT License.',
      copyright:
        'Copyright © <a href="https://github.com/jamesbrink">James Brink</a>',
    },

    editLink: {
      pattern: 'https://github.com/jamesbrink/koban/edit/main/website/:path',
      text: 'Edit this page on GitHub',
    },

    outline: {
      level: [2, 3],
    },
  },
})
