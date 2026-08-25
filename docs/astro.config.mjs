// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// https://astro.build/config
export default defineConfig({
	integrations: [
		starlight({
			title: 'nex',
			description: 'A lightweight, statically typed, compiled programming language.',
			social: [
				{ icon: 'github', label: 'GitHub', href: 'https://github.com/c4ist/nex' },
			],
			editLink: {
				baseUrl: 'https://github.com/c4ist/nex/edit/main/docs/src/content/docs/',
			},
			customCss: ['./src/styles/custom.css'],
			sidebar: [
				{ label: 'Getting Started', slug: 'getting-started' },
				{ label: 'Language Design', slug: 'language-design' },
				{
					label: 'Reference',
					items: [{ label: 'Lexical Structure', slug: 'reference/lexical-structure' }],
				},
				{
					label: 'Internals',
					items: [
						{ label: 'Architecture', slug: 'internals/architecture' },
						{ label: 'AST Coverage', slug: 'internals/ast-coverage' },
					],
				},
				{ label: 'Roadmap', slug: 'roadmap' },
			],
		}),
	],
});
