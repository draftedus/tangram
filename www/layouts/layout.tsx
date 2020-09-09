import backgroundImage from './background.svg'
import { Footer } from './footer'
import { Body, ComponentChildren, Head, PinwheelInfo, h, ui } from 'deps'

export type DocumentProps = {
	children?: ComponentChildren
	pinwheelInfo: PinwheelInfo
}

export function Document(props: DocumentProps) {
	return (
		<html lang="en">
			<head>
				<Head pinwheelInfo={props.pinwheelInfo} />
				<link
					as="font"
					crossOrigin="true"
					href="/jetbrainsmono.woff2"
					rel="preload"
					type="font/woff2"
				/>
				<link href="/ui.css" rel="stylesheet" />
				<link href="/www.css" rel="stylesheet" />
				<link href="/app.css" rel="stylesheet" />
				<title>{'Tangram'}</title>
				<meta
					content="All-In-One Machine Learning Toolkit for Developers"
					name="description"
				/>
			</head>
			<body>
				<Body pinwheelInfo={props.pinwheelInfo}>{props.children}</Body>
			</body>
		</html>
	)
}

type LayoutProps = {
	background?: boolean
	children?: ComponentChildren
	pinwheelInfo: PinwheelInfo
}

export function Layout(props: LayoutProps) {
	let wrapperStyle = {
		background: props.background && (`url(${backgroundImage})` as any),
	}
	return (
		<Document pinwheelInfo={props.pinwheelInfo}>
			<div class="layout" style={wrapperStyle}>
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

export enum LogoScheme {
	Multi,
	Solid,
}

type LogoProps = {
	class?: string
	color?: string
	colorScheme: LogoScheme
}

export function Logo(props: LogoProps) {
	let trapezoid
	let square
	let mediumTriangle
	let smallTriangle1
	let smallTriangle2
	let largeTriangle1
	let largeTriangle2
	if (props.colorScheme === LogoScheme.Multi) {
		trapezoid = 'var(--pink)'
		square = 'var(--yellow)'
		mediumTriangle = 'var(--teal)'
		smallTriangle1 = 'var(--purple)'
		smallTriangle2 = 'var(--indigo)'
		largeTriangle1 = 'var(--blue)'
		largeTriangle2 = 'var(--green)'
	} else {
		trapezoid = props.color
		square = props.color
		mediumTriangle = props.color
		smallTriangle1 = props.color
		smallTriangle2 = props.color
		largeTriangle1 = props.color
		largeTriangle2 = props.color
	}
	return (
		<svg class={props.class} height="100%" viewBox="0 0 200 200" width="100%">
			<desc>{'tangram'}</desc>
			<polygon
				fill={trapezoid}
				points="4 9.657 4 98.343 46 140.343 46 51.657"
			/>
			<polygon
				fill={square}
				points="100 105.657 55.657 150 100 194.343 144.343 150"
			/>
			<polygon fill={mediumTriangle} points="4 109.657 4 196 90.343 196" />
			<polygon fill={smallTriangle1} points="54 59.657 54 140.343 94.343 100" />
			<polygon
				fill={smallTriangle2}
				points="150 155.657 109.657 196 190.343 196"
			/>
			<polygon fill={largeTriangle1} points="190.343 4 9.657 4 100 94.343" />
			<polygon
				fill={largeTriangle2}
				points="196 9.657 105.657 100 196 190.343"
			/>
		</svg>
	)
}
