import { ui } from 'deps'

let featureWeightsChartContainer = document.getElementById('asdf')
if (!featureWeightsChartContainer) throw Error()
let featureWeightsChartPropsJson = featureWeightsChartContainer.dataset.props
if (!featureWeightsChartPropsJson) throw Error()
let options = JSON.parse(featureWeightsChartPropsJson)
let featureWeightsChart = ui.createBarChart(featureWeightsChartContainer)
featureWeightsChart.draw(options)
