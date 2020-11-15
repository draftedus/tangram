import "./styles.css"
import { PageInfo } from "@tangramhq/pinwheel"
import * as ui from "@tangramhq/ui"
import { PricingCards } from "common/pricing_cards"
import { renderPage } from "common/render"
import { PageLayout } from "layouts/page_layout"
import { h } from "preact"

export default (pageInfo: PageInfo) => {
	return renderPage(
		<PageLayout background={true} pageInfo={pageInfo}>
			<ui.S1>
				<ui.H1 center={true}>{"Pricing"}</ui.H1>
				<h2 class="pricing-subtitle">
					{"Training and prediction are free forever."}
					<br />
					{
						"Reporting, tuning, and monitoring are paid after a free trial. We are committed to simple public pricing."
					}
				</h2>
				<PricingCards
					enterpriseCta={
						<ui.Button color="var(--blue)" href="mailto:hello@tangramhq.com">
							{"Contact us"}
						</ui.Button>
					}
					freeCta={
						<ui.Button color="var(--indigo)" href="/docs">
							{"Read the Docs"}
						</ui.Button>
					}
					startupCta={
						<ui.Button
							color="var(--teal)"
							href="https://app.tangramhq.com/login"
						>
							{"Free Trial"}
						</ui.Button>
					}
					teamCta={
						<ui.Button
							color="var(--green)"
							href="https://app.tangramhq.com/login"
						>
							{"Free Trial"}
						</ui.Button>
					}
				/>
				<FAQs />
			</ui.S1>
		</PageLayout>,
	)
}

function FAQs() {
	return (
		<ui.S2>
			<ui.H2>{"What is Tangram?"}</ui.H2>
			<ui.P>
				{
					"Tangram makes it easy to train and deploy machine learning models. Developers can train a model on the command line and make predictions from any programming language. Product teams can then understand how the model works, tune it, and monitor it in production."
				}
			</ui.P>
			<ui.H2>{"Is my data sent to Tangram?"}</ui.H2>
			<ui.P>
				{
					"All training happens on the computer you run the CLI on and prediction happens in the same process as your code without any network calls. If you opt-in to logging, your production data will be sent to Tangram. If you are on the Enterprise plan and run the reporting and monitoring web app on your own servers, Tangram will never receive any of your production data."
				}
			</ui.P>
		</ui.S2>
	)
}
