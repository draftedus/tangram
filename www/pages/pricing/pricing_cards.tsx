import { cx } from '@tangramhq/pinwheel'
import * as ui from '@tangramhq/ui'
import { JSX, h } from 'preact'

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

export function PricingCards(props: PricingCardsProps) {
	return (
		<div class="pricing-cards-grid">
			<PricingCard
				color="var(--indigo)"
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
				color="var(--teal)"
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
				color="var(--green)"
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
				color="var(--blue)"
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

function PricingCard(props: PricingCardProps) {
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
