import { Footer } from './footer'
import './layout.css'
import { PageInfo } from '@tangramhq/pinwheel'
import * as ui from '@tangramhq/ui'
import { Document } from 'common/document'
import { Logo, LogoScheme } from 'common/logo'
import { ComponentChildren, h } from 'preact'

type LayoutProps = {
	background?: boolean
	children?: ComponentChildren
	pageInfo: PageInfo
}

export function Layout(props: LayoutProps) {
	return (
		<Document pageInfo={props.pageInfo}>
			<div class="layout">
				<header>
					<Topbar />
				</header>
				<main>{props.children}</main>
				<footer>
					<Footer />
				</footer>
			</div>
		</Document>
	)
}

export function Topbar() {
	let topbarItems: ui.TopbarItem[] = [
		{
			element: <GithubIconLink color="var(--text-color)" />,
			href: 'https://github.com/tangram-hq',
			title: 'Github',
		},
		{ href: '/benchmarks', title: 'Benchmarks' },
		{ href: '/pricing', title: 'Pricing' },
		{ href: '/docs/', title: 'Docs' },
		{ href: 'https://app.tangramhq.com', title: 'Sign In' },
		{
			element: <ui.Button href="/docs">{'Get Started'}</ui.Button>,
			href: '/docs',
			title: 'Get Started',
		},
	]
	return (
		<ui.Topbar
			activeTextColor="var(--accent-color)"
			backgroundColor="transparent"
			border="none"
			dropdownBackgroundColor="var(--surface-color)"
			foregroundColor="var(--text-color)"
			items={topbarItems}
			logo={<Logo colorScheme={LogoScheme.Multi} />}
			menuSeparatorColor="var(--text-color)"
			title="tangram"
		/>
	)
}

type GithubIconLinkProps = {
	color: string
}

function GithubIconLink(props: GithubIconLinkProps) {
	return (
		<ui.Link href="https://github.com/tangram-hq">
			<svg
				class="github-icon"
				fill={props.color}
				height="24"
				viewBox="0 0 16 16"
				width="24"
			>
				<desc>{'github'}</desc>
				<path d="M 8 0 C 3.58 0 0 3.58 0 8 C 0 11.54 2.29 14.53 5.47 15.59 C 5.87 15.66 6.02 15.42 6.02 15.21 C 6.02 15.02 6.01 14.39 6.01 13.72 C 4 14.09 3.48 13.23 3.32 12.78 C 3.23 12.55 2.84 11.84 2.5 11.65 C 2.22 11.5 1.82 11.13 2.49 11.12 C 3.12 11.11 3.57 11.7 3.72 11.94 C 4.44 13.15 5.59 12.81 6.05 12.6 C 6.12 12.08 6.33 11.73 6.56 11.53 C 4.78 11.33 2.92 10.64 2.92 7.58 C 2.92 6.71 3.23 5.99 3.74 5.43 C 3.66 5.23 3.38 4.41 3.82 3.31 C 3.82 3.31 4.49 3.1 6.02 4.13 C 6.66 3.95 7.34 3.86 8.02 3.86 C 8.7 3.86 9.38 3.95 10.02 4.13 C 11.55 3.09 12.22 3.31 12.22 3.31 C 12.66 4.41 12.38 5.23 12.3 5.43 C 12.81 5.99 13.12 6.7 13.12 7.58 C 13.12 10.65 11.25 11.33 9.47 11.53 C 9.76 11.78 10.01 12.26 10.01 13.01 C 10.01 14.08 10 14.94 10 15.21 C 10 15.42 10.15 15.67 10.55 15.59 C 13.71 14.53 16 11.53 16 8 C 16 3.58 12.42 0 8 0 Z" />
			</svg>
		</ui.Link>
	)
}
