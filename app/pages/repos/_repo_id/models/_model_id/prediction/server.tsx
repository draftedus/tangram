import {
	Column,
	EnumField,
	FieldType,
	Form,
	InnerType,
	NumberField,
	Props,
	TextField,
} from './props'
import './styles.css'
import { BarChart, BoxChart } from '@tangramhq/charts'
import * as ui from '@tangramhq/ui'
import { PredictionResult } from 'common/predict'
import { renderPage } from 'common/render'
import {
	EnumColumnToken,
	NumberColumnToken,
	TextColumnToken,
	UnknownColumnToken,
} from 'common/tokens'
import { ModelLayout, ModelSideNavItem } from 'layouts/model_layout'
import { Fragment, h } from 'preact'

export default function PredictPage(props: Props) {
	let inner
	switch (props.inner.type) {
		case InnerType.PredictionForm:
			inner = <PredictionForm form={props.inner.value.form} />
			break
		case InnerType.PredictionResult:
			inner = (
				<PredictionResult
					inputTable={props.inner.value.inputTable}
					prediction={props.inner.value.prediction}
				/>
			)
	}
	return renderPage(
		<ModelLayout
			info={props.modelLayoutInfo}
			pinwheelInfo={props.pinwheelInfo}
			selectedItem={ModelSideNavItem.Prediction}
		>
			{inner}
		</ModelLayout>,
	)
}

type PredictionFormProps = {
	form: Form
}

function PredictionForm(props: PredictionFormProps) {
	return (
		<ui.S1>
			<ui.H1>{'Prediction'}</ui.H1>
			<ui.Form>
				<div class="predict-form-items-wrapper">
					{props.form.fields.map(field => {
						switch (field.type) {
							case FieldType.Unknown:
								return <UnknownField field={field} />
							case FieldType.Number:
								return <NumberField field={field} />
							case FieldType.Enum:
								return <EnumField field={field} />
							case FieldType.Text:
								return <TextField field={field} />
						}
					})}
				</div>
				<div class="predict-form-buttons-wrapper">
					<ui.Button type="submit">{'Predict'}</ui.Button>
					<ui.Button color={ui.colors.yellow} type="reset">
						{'Reset Defaults'}
					</ui.Button>
				</div>
			</ui.Form>
		</ui.S1>
	)
}

type UnknownFieldProps = {
	field: Column
}

function UnknownField(props: UnknownFieldProps) {
	return (
		<>
			<div class="predict-field-wrapper">
				<div>
					<UnknownColumnToken />
				</div>
				<ui.TextField
					key={props.field.name}
					label={props.field.name}
					name={props.field.name}
					value={props.field.value}
				/>
			</div>
			<div></div>
		</>
	)
}

type NumberFieldProps = {
	field: NumberField
}

function NumberField(props: NumberFieldProps) {
	return (
		<>
			<div class="predict-field-wrapper">
				<div>
					<NumberColumnToken />
				</div>
				<ui.TextField
					key={props.field.name}
					label={props.field.name}
					name={props.field.name}
					value={props.field.value}
				/>
			</div>
			<div class="predict-column-chart-wrapper">
				<BoxChart
					class="column-chart"
					data={[
						{
							color: ui.colors.blue,
							data: [
								{
									label: props.field.name,
									x: 0,
									y: {
										max: props.field.max,
										min: props.field.min,
										p25: props.field.p25,
										p50: props.field.p50,
										p75: props.field.p75,
									},
								},
							],
							title: 'quartiles',
						},
					]}
					hideLegend={true}
					id={props.field.name}
					shouldDrawXAxisLabels={false}
				/>
			</div>
		</>
	)
}

type EnumFieldProps = {
	field: EnumField
}

function EnumField(props: EnumFieldProps) {
	return (
		<>
			<div class="predict-field-wrapper">
				<div>
					<EnumColumnToken />
				</div>
				<ui.SelectField
					key={props.field.name}
					label={props.field.name}
					name={props.field.name}
					options={props.field.options.map(option => ({
						text: option,
						value: option,
					}))}
					value={props.field.value ?? undefined}
				/>
			</div>
			<div class="predict-column-chart-wrapper">
				<BarChart
					class="column-chart"
					data={[
						{
							color: ui.colors.blue,
							data: props.field.histogram.map(([label, value], i) => ({
								label,
								x: i,
								y: value,
							})),
							title: 'histogram',
						},
					]}
					hideLegend={true}
					id={props.field.name}
					shouldDrawXAxisLabels={false}
				/>
			</div>
		</>
	)
}

type TextFieldProps = {
	field: TextField
}

function TextField(props: TextFieldProps) {
	return (
		<>
			<div class="predict-field-wrapper">
				<div>
					<TextColumnToken />
				</div>
				<ui.TextField
					key={props.field.name}
					label={props.field.name}
					name={props.field.name}
					value={props.field.value ?? undefined}
				/>
			</div>
			<div />
		</>
	)
}
