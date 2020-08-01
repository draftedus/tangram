import { JSX, css, cx, h, ui, useCss } from 'deps'

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

let gridCss = css({
	[`.pricing-cards-grid`]: {
		display: 'grid',
		gridGap: '0.5rem',
	},
	[ui.mobile]: {
		[`.pricing-cards-grid`]: {
			grid: 'repeat(4, auto)/ minmax(0, 1fr)',
		},
	},
	[ui.desktop]: {
		[`.pricing-cards-grid`]: {
			grid: 'auto / repeat(4, minmax(0, 1fr))',
		},
	},
})

export function PricingCards(props: PricingCardsProps) {
	useCss(gridCss)
	return (
		<div class="pricing-cards-grid">
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

let pricingGridCss = css({
	[`.pricing-card-grid`]: {
		alignContent: 'space-between',
		backgroundColor: ui.variables.colors.surface,
		border: `${ui.variables.border.width} solid ${ui.variables.colors.border}`,
		borderRadius: ui.variables.border.radius,
		display: 'grid',
		gridGap: '2rem',
		padding: '2rem',
	},
})

let pricingGridSelectedCss = css({
	[`pricing-card-grid-selected`]: {
		border: `${ui.variables.border.width} solid ${ui.variables.colors.accent}`,
	},
})

let pricingTopGridCss = css({
	[`.pricing-card-content-grid`]: {
		display: 'grid',
		gridGap: '1rem',
		justifyItems: 'center',
	},
})

let pricingCardPriceWrapper = css({
	[`.pricing-card-price-wrapper`]: {
		display: 'grid',
		gridGap: '0.5rem',
		justifyContent: 'center',
		justifyItems: 'center',
	},
})

let priceCss = css({
	[`.pricing-card-price`]: { fontSize: '2rem' },
})

let priceSubtitleCss = css({
	[`.pricing-card-subtitle`]: {
		color: ui.variables.colors.mutedText,
		fontSize: '1rem',
	},
})

let featureCss = css({
	[`.pricing-card-feature`]: {
		display: 'grid',
		gridAutoFlow: 'column',
		gridGap: '0.5rem',
		justifyContent: 'start',
		lineHeight: '1.5',
	},
	[`.pricing-card-feature:before`]: {
		color: ui.colors.blue,
		content: "'✔'",
	},
})

let featureListCss = css({
	[`.pricing-card-features-list`]: { display: 'grid', gridGap: '0.5rem' },
})

function PricingCard(props: PricingCardProps) {
	useCss(
		pricingGridCss,
		pricingGridSelectedCss,
		pricingTopGridCss,
		pricingCardPriceWrapper,
		priceCss,
		priceSubtitleCss,
		featureListCss,
		featureCss,
	)
	return (
		<div
			class={cx(
				'pricing-card-grid',
				props.selected && 'pricing-card-grid-selected',
			)}
		>
			<div class="pricing-card-content-grid">
				<div>
					<ui.Token color={props.color}>{props.title}</ui.Token>
				</div>
				<div class="pricing-card-price-wrapper">
					<div class="pricing-card-price">{props.price}</div>
					{props.priceSubtitle && (
						<div class="pricing-card-subtitle">{props.priceSubtitle}</div>
					)}
				</div>
				<div class="pricing-card-features-list">
					{props.features.map((feature, i) => (
						<div class="pricing-card-feature" key={i}>
							{feature}
						</div>
					))}
				</div>
			</div>
			<div>{props.cta}</div>
		</div>
	)
}
