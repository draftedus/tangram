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
			<ui.H2>{'How do I get started?'}</ui.H2>
			<ui.P>
				{
					'Tangram is open source and free to use as a single user. As engineers, we were annoyed every time we wanted to try a new tool but were met with a "contact us". We worked very hard to make Tangram incredibly easy to download and run. If anything is challenging in the set-up process let us know.'
				}
			</ui.P>
			<ui.H2>
				{'Do I need to be a machine learning engineer to use Tangram?'}
			</ui.H2>
			<ui.P>
				{
					'No! Tangram was designed to make machine learning accessible to all engineers. Just like you do not need to know how to implement a B-Tree map for a database, you do not need to know what one hot encoding is to create a machine learning model. There are some basic guidelines to keep in mind when choosing a dataset that will help you in making successful models. Our engineers are eager to help out so get in touch with any questions!'
				}
			</ui.P>
		</ui.S2>
	)
}
