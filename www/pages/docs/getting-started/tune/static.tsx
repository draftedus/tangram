import { PageInfo } from "@tangramhq/pinwheel"
import * as ui from "@tangramhq/ui"
import { renderPage } from "common/render"
import { DocsLayout, DocsPage } from "layouts/docs_layout"
import { h } from "preact"

import tuning from "./screenshots/tuning.png"
import tuningConfusionMatrix from "./screenshots/tuning_confusion_matrix.png"

export default (pageInfo: PageInfo) => {
	return renderPage(
		<DocsLayout pageInfo={pageInfo} selectedPage={DocsPage.Tune}>
			<ui.S1>
				<ui.H1>{"Getting Started"}</ui.H1>
				<ui.S2>
					<ui.H2>{"Tuning"}</ui.H2>
					<ui.P>
						{"Under "}
						<b>{"Tuning"}</b>
						{
							", we can tune the model to our preferred tradeoff between precision and recall. Since we would like to reduce false negatives, we want to increase recall."
						}
					</ui.P>
					<ui.Callout level={ui.Level.Info} title="Precision and Recall">
						{
							"Precision is the proportion of our model's positive predictions whose true value is actually positive. If your model is an email spam detector, you probably want high precision to minimize false positives: It is better to let a few spam emails into your inbox than to send an important email to the junk folder. Recall is the proportion of examples whose true value is positive that we predicted were positive. If your model is a credit card fraud detector, you probably want high recall to minimize false negatives: It is better to accidentally flag a few legitimate transactions than to let fraud through."
						}
					</ui.Callout>
					<ui.P>
						{
							"Using the slider, we can select a threshold that lowers false negatives without increasing false positives too much. As we drag the slider to change the threshold, we can see the metrics updating below it."
						}
					</ui.P>
					<ui.Img alt="tuning" src={tuning} />
					<ui.Img alt="tuning confusion matrix" src={tuningConfusionMatrix} />
					<ui.P>
						{
							"Once we are happy with a threshold, we can update our prediction code to use it. If we omit the threshold, the default value of 0.5 will be used."
						}
					</ui.P>
				</ui.S2>
			</ui.S1>
		</DocsLayout>,
	)
}
