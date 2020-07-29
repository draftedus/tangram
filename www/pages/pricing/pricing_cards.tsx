import { JSX, css, cssClass, cx, h, ui, useCss } from 'deps'

type PricingCardsProps = {
	enterpriseCta: JSX.Element
	enterpriseSelected?: boolean
	freeCta: JSX.Element
	freeSelected?: boolean
	startupCta: JSX.Element
	startupSelected?: boolean
	teamCta: JSX.Element
	teamSelected?: boolean
}

let gridClass = cssClass()
let gridCss = css({
	[`.${gridClass}`]: {
		display: 'grid',
		gridGap: '0.5rem',
	},
	[ui.mobile]: {
		[`.${gridClass}`]: {
			grid: 'repeat(4, auto)/ minmax(0, 1fr)',
		},
	},
	[ui.desktop]: {
		[`.${gridClass}`]: {
			grid: 'auto / repeat(4, minmax(0, 1fr))',
		},
	},
})

export function PricingCards(props: PricingCardsProps) {
	useCss(gridCss)
	return (
		<div class={gridClass}>
			<PricingCard
				color={ui.colors.indigo}
				cta={props.freeCta}
				features={[
					'Training with the CLI',
					'Prediction with the language libraries',
				]}
				price="$0"
				priceSubtitle="forever"
				selected={props.freeSelected}
				title="Free"
			/>
			<PricingCard
				color={ui.colors.teal}
				cta={props.startupCta}
				features={[
					'Everything in the Free plan',
					'Reporting',
					'Tuning',
					'Monitoring',
					'Single User',
				]}
				price="$99"
				priceSubtitle="per month"
				selected={props.startupSelected}
				title="Startup"
			/>
			<PricingCard
				color={ui.colors.green}
				cta={props.teamCta}
				features={[
					'Everything in the Free plan',
					'Reporting',
					'Tuning',
					'Monitoring',
					'Unlimited Users',
				]}
				price="$499"
				priceSubtitle="per user per month"
				selected={props.teamSelected}
				title="Team"
			/>
			<PricingCard
				color={ui.colors.blue}
				cta={props.enterpriseCta}
				features={[
					'Everything in the Team plan',
					'Run reporting and monitoring on your own servers',
				]}
				price="$999"
				priceSubtitle="per user per month"
				selected={props.enterpriseSelected}
				title="Enterprise"
			/>
		</div>
	)
}

type PricingCardProps = {
	color: string
	cta?: JSX.Element
	features: string[]
	price: string
	priceSubtitle?: string
	selected?: boolean
	title: string
}

let pricingGridClass = cssClass()
let pricingGridCss = css({
	[`.${pricingGridClass}`]: {
		alignContent: 'space-between',
		backgroundColor: ui.variables.colors.surface,
		border: `${ui.variables.border.width} solid ${ui.variables.colors.border}`,
		borderRadius: ui.variables.border.radius,
		display: 'grid',
		gridGap: '2rem',
		padding: '2rem',
	},
})

let pricingGridSelectedClass = cssClass()
let pricingGridSelectedCss = css({
	[`${pricingGridSelectedClass}`]: {
		border: `${ui.variables.border.width} solid ${ui.variables.colors.accent}`,
	},
})

let pricingTopGridClass = cssClass()
let pricingTopGridCss = css({
	[`.${pricingTopGridClass}`]: {
		display: 'grid',
		gridGap: '1rem',
		justifyItems: 'center',
	},
})

let priceClass = cssClass()
let priceCss = css({
	[`.${priceClass}`]: { fontSize: '2rem' },
})

let priceAndPriceSubtitleClass = cssClass()
let priceAndPriceSubtitleCss = css({
	[`.${priceAndPriceSubtitleClass}`]: {
		display: 'grid',
		gridGap: '0.5rem',
		justifyItems: 'center',
	},
})

let priceSubtitleClass = cssClass()
let priceSubtitleCss = css({
	[`.${priceSubtitleClass}`]: {
		color: ui.variables.colors.mutedText,
		fontSize: '1rem',
	},
})

let featureClass = cssClass()
let featureCss = css({
	[`.${featureClass}`]: {
		display: 'grid',
		gridAutoFlow: 'column',
		gridGap: '0.5rem',
		justifyContent: 'start',
		lineHeight: '1.5',
	},
	[`.${featureClass}:before`]: {
		color: ui.colors.blue,
		content: "'✔'",
	},
})

let featureListClass = cssClass()
let featureListCss = css({
	[`.${featureListClass}`]: { display: 'grid', gridGap: '0.5rem' },
})

function PricingCard(props: PricingCardProps) {
	useCss(pricingGridCss)
	useCss(pricingGridSelectedCss)
	useCss(pricingTopGridCss)
	useCss(priceAndPriceSubtitleCss)
	useCss(priceCss)
	useCss(priceSubtitleCss)
	useCss(featureListCss)
	useCss(featureCss)

	return (
		<div
			class={cx(pricingGridClass, props.selected && pricingGridSelectedClass)}
		>
			<div class={pricingTopGridClass}>
				<div>
					<ui.Token color={props.color}>{props.title}</ui.Token>
				</div>
				<div class={priceAndPriceSubtitleClass}>
					<div class={priceClass}>{props.price}</div>
					{props.priceSubtitle && (
						<div class={priceSubtitleClass}>{props.priceSubtitle}</div>
					)}
				</div>
				<div class={featureListClass}>
					{props.features.map((feature, i) => (
						<div class={featureClass} key={i}>
							{feature}
						</div>
					))}
				</div>
			</div>
			<div>{props.cta}</div>
		</div>
	)
}
