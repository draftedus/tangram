import { PricingCards } from './pricing_cards'
import { css, cssClass, h, ui, useCss } from 'deps'
import { PageLayout } from 'layouts/page_layout'

let subtitleClass = cssClass()
let subtitleCss = css({
	[`.${subtitleClass}`]: {
		fontSize: '1.5rem',
		fontWeight: 'normal',
		lineHeight: '1.5',
		margin: '0',
		textAlign: 'center',
	},
})

export default function Pricing() {
	useCss(subtitleCss)
	return (
		<PageLayout background>
			<ui.S1>
				<ui.H1 center>Pricing</ui.H1>
				<h2 class={subtitleClass}>
					Training and prediction are free forever.
					<br />
					Reporting, tuning, and monitoring are paid after a free trial.
				</h2>
				<PricingCards
					enterpriseCta={
						<ui.Button color={ui.colors.blue} href="mailto:hello@tangramhq.com">
							Contact us
						</ui.Button>
					}
					freeCta={
						<ui.Button color={ui.colors.indigo} href="/docs">
							Read the Docs
						</ui.Button>
					}
					startupCta={
						<ui.Button
							color={ui.colors.teal}
							href="https://app.tangramhq.com/login"
						>
							Free Trial
						</ui.Button>
					}
					teamCta={
						<ui.Button
							color={ui.colors.green}
							href="https://app.tangramhq.com/login"
						>
							Free Trial
						</ui.Button>
					}
				/>
				<FAQs />
			</ui.S1>
		</PageLayout>
	)
}

function FAQs() {
	return (
		<ui.S2>
			<ui.H2>What is Tangram?</ui.H2>
			<ui.P>
				Tangram makes it easy to train and deploy machine learning models.
				Developers can train a model on the command line and make predictions
				from any programming language. Product teams can then understand how the
				model works, tune it, and monitor it in production.
			</ui.P>
			<ui.H2>Is my data sent to Tangram?</ui.H2>
			<ui.P>
				All training happens on the computer you run the CLI on and prediction
				happens in the same process as your code without any network calls. If
				you opt-in to logging, your production data will be sent to Tangram. If
				you are on the Enterprise plan and run the reporting and monitoring app
				on your own servers, Tangram will never receive any of your production
				data.
			</ui.P>
		</ui.S2>
	)
}
