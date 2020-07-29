import { Explanations } from './explanations'
import explanations from './explanations.svg'
import { Introspection } from './introspection'
import introspection from './introspection.svg'
import { Prediction } from './prediction'
import prediction from './prediction.svg'
import { ProductionMetrics } from './production_metrics'
import productionMetrics from './production_metrics.svg'
import { ProductionStats } from './production_stats'
import productionStats from './production_stats.svg'
import { Training } from './training'
import training from './training.svg'
import { TrainingMetrics } from './training_metrics'
import trainingMetrics from './training_metrics.svg'
import { TrainingStats } from './training_stats'
import trainingStats from './training_stats.svg'
import { Tuning } from './tuning'
import tuning from './tuning.svg'
import { Client, css, cssClass, h, ui, useCss } from 'deps'
import { PageLayout } from 'layouts/page_layout'

export default function Home() {
	return (
		<PageLayout background>
			<ui.S1>
				<Hero />
				<Features />
			</ui.S1>
		</PageLayout>
	)
}

let wrapperClass = cssClass()
let wrapperCss = css({
	[`.${wrapperClass}`]: { color: ui.variables.colors.text },
})

let innerClass = cssClass()
let innerCss = css({
	[`.${innerClass}`]: {
		display: 'grid',
		grid: 'auto auto / minmax(0, 1fr)',
	},
	[ui.mobile]: {
		[`.${innerClass}`]: {
			gridRowGap: '1rem',
		},
	},
	[ui.desktop]: {
		[`.${innerClass}`]: {
			gridRowGap: '2rem',
		},
	},
})

let titleClass = cssClass()
let titleCss = css({
	[`.${titleClass}`]: {
		margin: '0',
		textAlign: 'center',
	},
	[ui.mobile]: {
		[`.${titleClass}`]: {
			fontSize: '2rem',
		},
	},
	[ui.desktop]: {
		[`.${titleClass}`]: {
			fontSize: '3rem',
		},
	},
})

let subtitleClass = cssClass()
let subtitleCss = css({
	[`.${subtitleClass}`]: {
		lineHeight: '1.5',
		textAlign: 'center',
	},
	[ui.mobile]: {
		[`.${subtitleClass}`]: {
			fontSize: '1.25rem',
		},
	},
	[ui.desktop]: {
		[`.${subtitleClass}`]: {
			fontSize: '1.5rem',
		},
	},
})

function Hero() {
	useCss(wrapperCss)
	useCss(innerCss)
	useCss(titleCss)
	useCss(subtitleCss)

	return (
		<div class={wrapperClass}>
			<div class={innerClass}>
				<h1 class={titleClass}>Machine Learning for Developers</h1>
				<div class={subtitleClass}>
					Train a model on the command line and make predictions in any
					programming language. Understand how your model works, tune it, and
					monitor it in production.
				</div>
				<Video />
			</div>
		</div>
	)
}

let aspectRatio = 16 / 9
let placeholderClass = cssClass()
let placeholderCss = css({
	[`.${placeholderClass}`]: {
		backgroundColor: '#282c34',
		borderRadius: ui.variables.border.radius,
		overflow: 'hidden',
		paddingTop: `${(1 / aspectRatio) * 100}%`,
		position: 'relative',
		width: '100%',
	},
})

function Video() {
	useCss(wrapperCss)
	useCss(placeholderCss)
	let videoStyle = {
		border: 'none',
		bottom: '0',
		height: '100%',
		left: '0',
		outline: 'none',
		position: 'absolute',
		right: '0',
		top: '0',
		width: '100%',
	}
	return (
		<div class={wrapperClass}>
			<div class={placeholderClass}>
				<iframe
					allowFullScreen
					src="https://player.vimeo.com/video/385352664"
					style={videoStyle}
					title="tangram video"
				/>
			</div>
		</div>
	)
}

let featuresGridClass = cssClass()
let featuresGridCss = css({
	[`.${featuresGridClass}`]: {
		display: 'grid',
	},
	[ui.mobile]: {
		[`.${featuresGridClass}`]: {
			grid: 'auto / minmax(0, 1fr)',
			gridRowGap: '2rem',
		},
	},
	[ui.desktop]: {
		[`.${featuresGridClass}`]: {
			grid: 'auto / minmax(0, 2fr) minmax(0, 1fr)',
			gridAutoFlow: 'dense',
			gridColumnGap: '4rem',
			gridRowGap: '4rem',
		},
	},
})

let stepClass = cssClass()
let stepCss = css({
	[`.${stepClass}`]: {
		alignContent: 'start',
		display: 'grid',
		gridGap: '0.5rem',
	},
	[ui.mobile]: {
		[`.${stepClass}`]: {
			gridColumn: '1',
		},
	},
	[ui.desktop]: {
		[`.${stepClass}`]: {
			gridColumn: '2',
		},
	},
})

let stepTitleWrapperClass = cssClass()
let stepTitleWrapperCss = css({
	[`.${stepTitleWrapperClass}`]: {
		alignItems: 'center',
		display: 'grid',
		grid: 'auto / auto auto',
		gridGap: '0.5rem',
		justifyContent: 'start',
	},
})

let stepIconClass = cssClass()
let stepIconCss = css({
	[`.${stepIconClass}`]: { height: '2rem', width: '2rem' },
})

let stepTitleClass = cssClass()
let stepTitleCss = css({
	[`.${stepTitleClass}`]: {
		fontSize: '1.5rem',
		fontWeight: 'bold',
		margin: '0',
	},
})

function Features() {
	useCss(featuresGridCss)
	useCss(stepCss)
	useCss(stepTitleWrapperCss)
	useCss(stepIconCss)
	useCss(stepTitleCss)

	return (
		<div class={featuresGridClass}>
			<div class={stepClass}>
				<div class={stepTitleWrapperClass}>
					<img alt="training icon" class={stepIconClass} src={training} />
					<div class={stepTitleClass}>Training</div>
				</div>
				<ui.S2>
					<ui.P>
						Train a model using the Tangram command line tool. Just pass a CSV
						file with your data and the name of the column you want to predict.
						Tangram automatically converts text, numeric and categorical data in
						into features machine learning models understand, trains linear and
						gradient boosted decision trees across a large hyperparameter grid
						and finally chooses the model with the best performance.
					</ui.P>
					<ui.P>
						<b>
							Your data stays secure because all training happens on your laptop
							or your own server.
						</b>
					</ui.P>
				</ui.S2>
			</div>
			<Client component={Training} id="training" props={{}} />
			<div class={stepClass}>
				<div class={stepTitleWrapperClass}>
					<img alt="prediction icon" class={stepIconClass} src={prediction} />
					<div class={stepTitleClass}>Prediction</div>
				</div>
				<ui.S2>
					<ui.P>
						Make predictions from your favorite programming language using the{' '}
						<ui.InlineCode>.tangram</ui.InlineCode> file generated by the
						command line tool.
					</ui.P>
					<ui.P>
						<b>
							Prediction happens locally, as opposed to calling an external API,
							so it's super fast and your data stays secure.
						</b>
					</ui.P>
					<ui.P>
						Libraries are available for Go, JavaScript, Python, and Ruby.
						Support for C/C++, C#, Java, PHP, and Rust is coming soon.
					</ui.P>
				</ui.S2>
			</div>
			<div>
				<Prediction />
			</div>
			<div class={stepClass}>
				<div class={stepTitleWrapperClass}>
					<img
						alt="training stats icon"
						class={stepIconClass}
						src={trainingStats}
					/>
					<div class={stepTitleClass}>Training Stats</div>
				</div>
				<ui.S2>
					<ui.P>
						Want to learn more about your model? Upload it to the Tangram
						reporting and monitoring app.
					</ui.P>
					<ui.P>
						View box plots showing the distribution of the numeric columns in
						your dataset, bar charts for categorical columns and word token
						histograms for text columns.
					</ui.P>
					<ui.P>
						Dirty data? Tangram calculates the number of missing values in each
						column so you can easily spot issues in your dataset.
					</ui.P>
					<ui.P>
						<b>
							The reporting and monitoring app can be run by us in the cloud or
							by you on your own servers.
						</b>
					</ui.P>
				</ui.S2>
			</div>
			<Client component={TrainingStats} id="training-stats" props={{}} />
			<div class={stepClass}>
				<div class={stepTitleWrapperClass}>
					<img
						alt="training metrics icon"
						class={stepIconClass}
						src={trainingMetrics}
					/>
					<div class={stepTitleClass}>Training Metrics</div>
				</div>
				<ui.S2>
					<ui.P>
						View metrics showing how your model performed on the test set.
					</ui.P>
					<ui.P>
						Tangram takes care of splitting your csv into a train set used to
						train your model and a test set used to evaluate your model.
					</ui.P>
					<ui.P>
						View common evaluation metrics like accuracy, precision, f1 scores
						and pr and roc curves.
					</ui.P>
					<ui.P>
						Find out where your model is making mistakes by viewing per-class
						confusion matrices.
					</ui.P>
				</ui.S2>
			</div>
			<Client component={TrainingMetrics} id="training-metrics" props={{}} />
			<div class={stepClass}>
				<div class={stepTitleWrapperClass}>
					<img
						alt="introspection icon"
						class={stepIconClass}
						src={introspection}
					/>
					<div class={stepTitleClass}>Introspection</div>
				</div>
				<ui.S2>
					<ui.P>
						Learn more about how your model is making predictions by viewing the
						features that were most important to your model.
					</ui.P>
					<ui.P>
						View feature importance scores for gradient boosted decision tree
						models and raw model weights for linear models.
					</ui.P>
				</ui.S2>
			</div>
			<Client component={Introspection} id="introspection" props={{}} />
			<div class={stepClass}>
				<div class={stepTitleWrapperClass}>
					<img
						alt="explainability icon"
						class={stepIconClass}
						src={explanations}
					/>
					<div class={stepTitleClass}>Explanations</div>
				</div>
				<ui.S2>
					<ui.P>
						Figure out why your model made a particular prediction. Features
						with green/positive contributions push the probability of the class
						higher while features with red/negative contributions push the
						probability of the class lower.
					</ui.P>
					<ui.P>
						In this example, the baseline probability of being "positive" for
						heart disease is 51%. The patient's high cholesterol and low
						exercise max heart rate increased the probability that they have
						heart disease while their normal thallium stress test decreased the
						probability of heart disease lower.
					</ui.P>
				</ui.S2>
			</div>
			<Client component={Explanations} id="explanations" props={{}} />
			<div class={stepClass}>
				<div class={stepTitleWrapperClass}>
					<img alt="tuning icon" class={stepIconClass} src={tuning} />
					<div class={stepTitleClass}>Tuning</div>
				</div>
				<ui.S2>
					<ui.P>
						Tune your model to your preferred tradeoff between precision and
						recall.
					</ui.P>
					<ui.P>
						For example, if your model is a fraud detector, you probably want
						high recall to minimize false negatives. On the other hand, if your
						model is an email spam detector, you probably want high precision to
						minimize false positives.
					</ui.P>
					<ui.P>
						To use your selected threshold, just update the predict call in your
						code.
					</ui.P>
				</ui.S2>
			</div>
			<Client component={Tuning} id="tuning" props={{}} />
			<div class={stepClass}>
				<div class={stepTitleWrapperClass}>
					<img
						alt="production stats icon"
						class={stepIconClass}
						src={productionStats}
					/>
					<div class={stepTitleClass}>Production Stats</div>
				</div>
				<ui.S2>
					<ui.P>
						When your model is in production, you want to make sure that the
						data coming in is similar to what you trained on. You can opt in to
						logging by calling <ui.InlineCode>logPrediction</ui.InlineCode> in
						your code.
					</ui.P>
					<ui.P>
						Then you can compare production stats with training stats to make
						sure the distributions of each of the columns in production is
						similar to what you trained on.
					</ui.P>
					<ui.P>
						Tangram automatically alerts you when it detects issues in your
						production data.
					</ui.P>
				</ui.S2>
			</div>
			<Client component={ProductionStats} id="production-stats" props={{}} />
			<div class={stepClass}>
				<div class={stepTitleWrapperClass}>
					<img
						alt="production metrics icon"
						class={stepIconClass}
						src={productionMetrics}
					/>
					<div class={stepTitleClass}>Production Metrics</div>
				</div>
				<ui.S2>
					<ui.P>
						Later on, you may find out the true value for what you were trying
						to predict. In your code, you can call{' '}
						<ui.InlineCode>logTrueValue</ui.InlineCode>.
					</ui.P>
					<ui.P>
						Then you can view production metrics to ensure your model is as
						accurate in production as it was in training.
					</ui.P>
					<ui.P>
						Sometimes datasets change over time. View metrics over time to
						ensure your model is performing as well as when you first trained
						it.
					</ui.P>
				</ui.S2>
			</div>
			<Client
				component={ProductionMetrics}
				id="production-metrics"
				props={{}}
			/>
		</div>
	)
}
