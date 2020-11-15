import { hydrateLineChart } from "@tangramhq/charts"
import { bootClassSelect } from "common/class_select_field"
import { bootDateWindowSelect } from "common/date_window_select_field"

bootDateWindowSelect()
bootClassSelect()
hydrateLineChart("precision_intervals")
hydrateLineChart("recall_intervals")
hydrateLineChart("f1_intervals")
