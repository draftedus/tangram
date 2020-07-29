export type ComparisonHistogramValue = {
	productionCount: number
	productionFraction: number
	trainingCount: number
	trainingFraction: number
}

export type ComparisonHistogram = Array<[string, ComparisonHistogramValue]>
