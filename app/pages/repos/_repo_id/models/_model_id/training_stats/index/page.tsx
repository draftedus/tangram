import { MetricsRow } from 'common/metrics_row'
import {
	EnumColumnToken,
	NumberColumnToken,
	TextColumnToken,
	UnknownColumnToken,
} from 'common/tokens'
import { Fragment, h, ui } from 'deps'
import { ModelLayout, ModelLayoutProps } from 'layouts/model_layout'

let description = `The following are statistics for the columns in your dataset. A column is a Number column if every non-null value in the column parses as a finite floating point number. A column is an Enum column if it is not a Number column and the count of unique non-null values is less than 100. A column is a Text column if it is neither a Number column nor an Enum column.`

export type Props = {
	columnCount: number
	columnStats: Array<{
		columnType?: ColumnType
		invalidCount?: number
		max: number | null
		mean: number | null
		min: number | null
		name: string
		std: number | null
		uniqueCount: number | null
		variance: number | null
	}>
	modelId: string
	modelLayoutProps: ModelLayoutProps
	rowCount: number
	targetColumnStats: {
		columnType?: ColumnType
		invalidCount?: number
		max: number | null
		mean: number | null
		min: number | null
		name: string
		std: number | null
		uniqueCount: number | null
		variance: number | null
	}
}

export enum ColumnType {
	Unknown = 'unknown',
	Number = 'number',
	Enum = 'enum',
	Text = 'text',
}

export default function TrainingStatsIndexPage(props: Props) {
	return (
		<ModelLayout {...props.modelLayoutProps}>
			<ui.S1>
				<ui.H1>Training Stats</ui.H1>
				<ui.S2>
					<ui.H2>Target Column</ui.H2>
					<ui.Table width="100%">
						<ui.TableHeader>
							<ui.TableRow>
								<ui.TableHeaderCell>Column</ui.TableHeaderCell>
								<ui.TableHeaderCell>Type</ui.TableHeaderCell>
								<ui.TableHeaderCell>Unique Count</ui.TableHeaderCell>
								{props.targetColumnStats.columnType == ColumnType.Number && (
									<Fragment>
										<ui.TableHeaderCell>Min</ui.TableHeaderCell>
										<ui.TableHeaderCell>Max</ui.TableHeaderCell>
										<ui.TableHeaderCell>Mean</ui.TableHeaderCell>
										<ui.TableHeaderCell>Std</ui.TableHeaderCell>
									</Fragment>
								)}
							</ui.TableRow>
						</ui.TableHeader>
						<ui.TableBody>
							<ui.TableRow key={props.targetColumnStats.name}>
								<ui.TableCell>
									<ui.Link href={`./columns/${props.targetColumnStats.name}`}>
										{props.targetColumnStats.name}
									</ui.Link>
								</ui.TableCell>
								<ui.TableCell>
									{props.targetColumnStats.columnType === ColumnType.Number ? (
										<NumberColumnToken />
									) : props.targetColumnStats.columnType === ColumnType.Enum ? (
										<EnumColumnToken />
									) : props.targetColumnStats.columnType === ColumnType.Text ? (
										<TextColumnToken />
									) : null}
								</ui.TableCell>
								<ui.TableCell>
									{props.targetColumnStats.uniqueCount}
								</ui.TableCell>
								{props.targetColumnStats.columnType == ColumnType.Number && (
									<Fragment>
										<ui.TableCell>{props.targetColumnStats.min}</ui.TableCell>
										<ui.TableCell>{props.targetColumnStats.max}</ui.TableCell>
										<ui.TableCell>{props.targetColumnStats.mean}</ui.TableCell>
										<ui.TableCell>{props.targetColumnStats.std}</ui.TableCell>
									</Fragment>
								)}
							</ui.TableRow>
						</ui.TableBody>
					</ui.Table>
				</ui.S2>
				<ui.S2>
					<ui.H2>Columns</ui.H2>
					<ui.P>{description}</ui.P>
					<MetricsRow>
						<ui.Card>
							<ui.NumberChart title="Rows" value={props.rowCount.toString()} />
						</ui.Card>
						<ui.Card>
							<ui.NumberChart
								title="Columns"
								value={props.columnCount.toString()}
							/>
						</ui.Card>
					</MetricsRow>
					<ui.Table width="100%">
						<ui.TableHeader>
							<ui.TableRow>
								<ui.TableHeaderCell>Column</ui.TableHeaderCell>
								<ui.TableHeaderCell>Type</ui.TableHeaderCell>
								<ui.TableHeaderCell>Unique Values Count</ui.TableHeaderCell>
								<ui.TableHeaderCell>Null Count</ui.TableHeaderCell>
								<ui.TableHeaderCell>Min</ui.TableHeaderCell>
								<ui.TableHeaderCell>Max</ui.TableHeaderCell>
								<ui.TableHeaderCell>Mean</ui.TableHeaderCell>
								<ui.TableHeaderCell>Std</ui.TableHeaderCell>
							</ui.TableRow>
						</ui.TableHeader>
						<ui.TableBody>
							{props.columnStats.map(columnStats => {
								return (
									<ui.TableRow key={columnStats.name}>
										<ui.TableCell>
											{columnStats.columnType === ColumnType.Unknown ? (
												columnStats.name
											) : (
												<ui.Link href={`./columns/${columnStats.name}`}>
													{columnStats.name}
												</ui.Link>
											)}
										</ui.TableCell>
										<ui.TableCell>
											{columnStats.columnType === ColumnType.Number ? (
												<NumberColumnToken />
											) : columnStats.columnType === ColumnType.Enum ? (
												<EnumColumnToken />
											) : columnStats.columnType === ColumnType.Text ? (
												<TextColumnToken />
											) : columnStats.columnType === ColumnType.Unknown ? (
												<UnknownColumnToken />
											) : null}
										</ui.TableCell>
										<ui.TableCell>{columnStats.uniqueCount}</ui.TableCell>
										<ui.TableCell>{columnStats.invalidCount}</ui.TableCell>
										<ui.TableCell>
											{ui.formatNumber(columnStats.min)}
										</ui.TableCell>
										<ui.TableCell>
											{ui.formatNumber(columnStats.max)}
										</ui.TableCell>
										<ui.TableCell>
											{ui.formatNumber(columnStats.mean)}
										</ui.TableCell>
										<ui.TableCell>
											{ui.formatNumber(columnStats.std)}
										</ui.TableCell>
									</ui.TableRow>
								)
							})}
						</ui.TableBody>
					</ui.Table>
				</ui.S2>
			</ui.S1>
		</ModelLayout>
	)
}
