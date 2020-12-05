import "./tuning.css"
import * as ui from "@tangramhq/ui"
import { h } from "preact"
import { useState } from "preact/hooks"

export function Tuning() {
	let [thresholdIndex, setThresholdIndex] = useState(9)

	function formatter(thresholdIndex: number) {
		return (thresholdIndex * 0.05 + 0.05).toPrecision(2)
	}

	let thresholdString = formatter(thresholdIndex)
	let metrics = data[thresholdIndex]
	if (metrics === undefined) throw Error()
	let accuracy = ui.formatPercent(metrics.accuracy, 2)
	let precision = ui.formatPercent(metrics.precision, 2)
	let recall = ui.formatPercent(metrics.recall, 2)

	return (
		<ui.Window>
			<div class="tuning-grid">
				<ui.Slider
					color="var(--blue)"
					max={18}
					min={0}
					onChange={setThresholdIndex}
					step={1}
					value={thresholdIndex}
					valueFormatter={formatter}
				/>
				<div class="tuning-number-chart-grid">
					<div style={{ gridArea: "accuracy" }}>
						<ui.Card>
							<ui.NumberChart title="Accuracy" value={accuracy} />
						</ui.Card>
					</div>
					<div style={{ gridArea: "precision" }}>
						<ui.Card>
							<ui.NumberChart title="Precision" value={precision} />
						</ui.Card>
					</div>
					<div style={{ gridArea: "recall" }}>
						<ui.Card>
							<ui.NumberChart title="Recall" value={recall} />
						</ui.Card>
					</div>
				</div>
				<ui.Code
					code={`// Update your code to use the selected threshold.\nmodel.predict(input, { threshold: ${thresholdString} })`}
					language={ui.Language.JavaScript}
				/>
			</div>
		</ui.Window>
	)
}

type ThresholdMetrics = {
	accuracy: number
	f1Score: number
	falseNegatives: number
	falsePositives: number
	precision: number
	recall: number
	threshold: number
	trueNegatives: number
	truePositives: number
}

let data: ThresholdMetrics[] = [
	{
		accuracy: 0.569629967212677,
		f1Score: 0.529461145401001,
		falseNegatives: 17,
		falsePositives: 2786,
		precision: 0.3614485561847687,
		recall: 0.9893350005149841,
		threshold: 0.04999999701976776,
		trueNegatives: 2133,
		truePositives: 1577,
	},
	{
		accuracy: 0.7005987763404846,
		f1Score: 0.6121718287467957,
		falseNegatives: 55,
		falsePositives: 1895,
		precision: 0.4481654167175293,
		recall: 0.9654955863952637,
		threshold: 0.09999999403953552,
		trueNegatives: 3024,
		truePositives: 1539,
	},
	{
		accuracy: 0.7595577836036682,
		f1Score: 0.6577797532081604,
		falseNegatives: 89,
		falsePositives: 1477,
		precision: 0.5046948194503784,
		recall: 0.9441656470298767,
		threshold: 0.14999999105930328,
		trueNegatives: 3442,
		truePositives: 1505,
	},
	{
		accuracy: 0.8040841221809387,
		f1Score: 0.6920849680900574,
		falseNegatives: 160,
		falsePositives: 1116,
		precision: 0.5623529553413391,
		recall: 0.8996235728263855,
		threshold: 0.19999998807907104,
		trueNegatives: 3803,
		truePositives: 1434,
	},
	{
		accuracy: 0.8238906860351562,
		f1Score: 0.7026185989379883,
		falseNegatives: 239,
		falsePositives: 908,
		precision: 0.5987626910209656,
		recall: 0.8500627279281616,
		threshold: 0.25,
		trueNegatives: 4011,
		truePositives: 1355,
	},
	{
		accuracy: 0.8430830836296082,
		f1Score: 0.7068272829055786,
		falseNegatives: 362,
		falsePositives: 660,
		precision: 0.6511628031730652,
		recall: 0.7728983759880066,
		threshold: 0.29999998211860657,
		trueNegatives: 4259,
		truePositives: 1232,
	},
	{
		accuracy: 0.8486104607582092,
		f1Score: 0.6943583488464355,
		falseNegatives: 474,
		falsePositives: 512,
		precision: 0.686274528503418,
		recall: 0.702634871006012,
		threshold: 0.3499999940395355,
		trueNegatives: 4407,
		truePositives: 1120,
	},
	{
		accuracy: 0.8542914390563965,
		f1Score: 0.6903751492500305,
		falseNegatives: 536,
		falsePositives: 413,
		precision: 0.7192386388778687,
		recall: 0.663739025592804,
		threshold: 0.3999999761581421,
		trueNegatives: 4506,
		truePositives: 1058,
	},
	{
		accuracy: 0.8558267951011658,
		f1Score: 0.6749740839004517,
		falseNegatives: 619,
		falsePositives: 320,
		precision: 0.7528957724571228,
		recall: 0.6116687655448914,
		threshold: 0.44999998807907104,
		trueNegatives: 4599,
		truePositives: 975,
	},
	{
		accuracy: 0.8567480444908142,
		f1Score: 0.6591158509254456,
		falseNegatives: 692,
		falsePositives: 241,
		precision: 0.7891513705253601,
		recall: 0.5658720135688782,
		threshold: 0.5,
		trueNegatives: 4678,
		truePositives: 902,
	},
	{
		accuracy: 0.8567480444908142,
		f1Score: 0.6467247009277344,
		falseNegatives: 740,
		falsePositives: 193,
		precision: 0.8156638145446777,
		recall: 0.5357590913772583,
		threshold: 0.550000011920929,
		trueNegatives: 4726,
		truePositives: 854,
	},
	{
		accuracy: 0.8565945029258728,
		f1Score: 0.6311216354370117,
		falseNegatives: 795,
		falsePositives: 139,
		precision: 0.8518123626708984,
		recall: 0.501254677772522,
		threshold: 0.5999999642372131,
		trueNegatives: 4780,
		truePositives: 799,
	},
	{
		accuracy: 0.8486104607582092,
		f1Score: 0.5881369709968567,
		falseNegatives: 890,
		falsePositives: 96,
		precision: 0.8799999952316284,
		recall: 0.4416562020778656,
		threshold: 0.6499999761581421,
		trueNegatives: 4823,
		truePositives: 704,
	},
	{
		accuracy: 0.8401657938957214,
		f1Score: 0.5383592247962952,
		falseNegatives: 987,
		falsePositives: 54,
		precision: 0.9183055758476257,
		recall: 0.38080301880836487,
		threshold: 0.699999988079071,
		trueNegatives: 4865,
		truePositives: 607,
	},
	{
		accuracy: 0.8289574980735779,
		f1Score: 0.47452831268310547,
		falseNegatives: 1091,
		falsePositives: 23,
		precision: 0.9562737345695496,
		recall: 0.3155583441257477,
		threshold: 0.75,
		trueNegatives: 4896,
		truePositives: 503,
	},
	{
		accuracy: 0.8166743516921997,
		f1Score: 0.40597015619277954,
		falseNegatives: 1186,
		falsePositives: 8,
		precision: 0.9807692170143127,
		recall: 0.2559598386287689,
		threshold: 0.7999999523162842,
		trueNegatives: 4911,
		truePositives: 408,
	},
	{
		accuracy: 0.8120681643486023,
		f1Score: 0.37931033968925476,
		falseNegatives: 1220,
		falsePositives: 4,
		precision: 0.9894179701805115,
		recall: 0.23462985455989838,
		threshold: 0.8499999642372131,
		trueNegatives: 4915,
		truePositives: 374,
	},
	{
		accuracy: 0.8051589131355286,
		f1Score: 0.34077921509742737,
		falseNegatives: 1266,
		falsePositives: 3,
		precision: 0.9909365773200989,
		recall: 0.20577163994312286,
		threshold: 0.8999999761581421,
		trueNegatives: 4916,
		truePositives: 328,
	},
	{
		accuracy: 0.7557193040847778,
		f1Score: 0.0037570444401353598,
		falseNegatives: 1591,
		falsePositives: 0,
		precision: 1,
		recall: 0.001882057753391564,
		threshold: 0.949999988079071,
		trueNegatives: 4919,
		truePositives: 3,
	},
]
