import "./training_metrics.css"
import { LineChart, LineStyle, PointStyle } from "@tangramhq/charts"
import * as ui from "@tangramhq/ui"
import { h } from "preact"

export function TrainingMetrics() {
	let rocData = [
		{
			color: ui.colors.blue,
			data: data.rocData,
			pointStyle: PointStyle.Hidden,
			title: "ROC",
		},
		{
			color: ui.colors.gray,
			data: [
				{ x: 0, y: 0 },
				{ x: 1, y: 1 },
			],
			lineStyle: LineStyle.Dashed,
			pointStyle: PointStyle.Hidden,
			title: "Reference",
		},
	]
	let prData = [
		{
			color: ui.colors.blue,
			data: data.prData,
			pointStyle: PointStyle.Circle,
			title: "PR",
		},
	]
	return (
		<ui.Window>
			<div class="training-metrics-wrapper">
				<div style={{ gridArea: "accuracy" }}>
					<ui.Card>
						<ui.NumberComparisonChart
							colorA="var(--gray)"
							colorB="var(--blue)"
							title="Accuracy"
							valueA={data.baselineAccuracy}
							valueATitle="Baseline"
							valueB={data.accuracy}
							valueBTitle="Training"
							valueFormatter={value => ui.formatPercent(value, 2)}
						/>
					</ui.Card>
				</div>
				<div style={{ gridArea: "pr" }}>
					<ui.Card>
						<LineChart
							data={prData}
							hideLegend={true}
							title="PR Curve"
							xAxisTitle="Precision"
							xMax={1}
							xMin={0}
							yAxisTitle="Recall"
							yMax={1}
							yMin={0}
						/>
					</ui.Card>
				</div>
				<div style={{ gridArea: "roc" }}>
					<ui.Card>
						<LineChart
							data={rocData}
							hideLegend={true}
							title="ROC Curve"
							xAxisTitle="FPR"
							xMax={1}
							xMin={0}
							yAxisTitle="TPR"
							yMax={1}
							yMin={0}
						/>
					</ui.Card>
				</div>
			</div>
		</ui.Window>
	)
}

let data = {
	accuracy: 0.8567480444908142,
	aucRoc: 0.9171663522720337,
	baselineAccuracy: 0.7552587389945984,
	classMetrics: [
		{
			accuracy: 0.8567480444908142,
			className: "<=50K",
			f1Score: 0.909320592880249,
			falseNegatives: 241,
			falsePositives: 692,
			precision: 0.871135950088501,
			recall: 0.951006293296814,
			trueNegatives: 902,
			truePositives: 4678,
		},
		{
			accuracy: 0.8567480444908142,
			className: ">50K",
			f1Score: 0.6591158509254456,
			falseNegatives: 692,
			falsePositives: 241,
			precision: 0.7891513705253601,
			recall: 0.5658720135688782,
			trueNegatives: 4678,
			truePositives: 902,
		},
	],
	classes: ["<=50K", ">50K"],
	loss: 0.309477,
	prData: [
		{ x: 0.001829640124924481, y: 1 },
		{ x: 0.10225655883550644, y: 0.9980158805847168 },
		{ x: 0.18682658672332764, y: 0.9967461824417114 },
		{ x: 0.24273225665092468, y: 0.9974937438964844 },
		{ x: 0.29335230588912964, y: 0.9965469837188721 },
		{ x: 0.36043912172317505, y: 0.9943915009498596 },
		{ x: 0.4088229238986969, y: 0.9930863976478577 },
		{ x: 0.4336247146129608, y: 0.992093026638031 },
		{ x: 0.46696484088897705, y: 0.9913681745529175 },
		{ x: 0.50518399477005, y: 0.990829348564148 },
		{ x: 0.5222606062889099, y: 0.9895994067192078 },
		{ x: 0.5364911556243896, y: 0.9887598156929016 },
		{ x: 0.5495019555091858, y: 0.9875776171684265 },
		{ x: 0.5696280002593994, y: 0.9862724542617798 },
		{ x: 0.5840618014335632, y: 0.9852537512779236 },
		{ x: 0.5917869210243225, y: 0.9844436645507812 },
		{ x: 0.6015450358390808, y: 0.9830564856529236 },
		{ x: 0.614759087562561, y: 0.9821370840072632 },
		{ x: 0.6243138909339905, y: 0.98209148645401 },
		{ x: 0.6338686943054199, y: 0.981120228767395 },
		{ x: 0.6476926207542419, y: 0.9794036149978638 },
		{ x: 0.6568408012390137, y: 0.9779055714607239 },
		{ x: 0.6633462309837341, y: 0.9763614535331726 },
		{ x: 0.6694450378417969, y: 0.9759928584098816 },
		{ x: 0.6753405332565308, y: 0.9756240844726562 },
		{ x: 0.6824557781219482, y: 0.9755885004997253 },
		{ x: 0.69160395860672, y: 0.9747850894927979 },
		{ x: 0.699735701084137, y: 0.9747946858406067 },
		{ x: 0.7031916975975037, y: 0.9738175868988037 },
		{ x: 0.7223013043403625, y: 0.9720930457115173 },
		{ x: 0.72941654920578, y: 0.9715678095817566 },
		{ x: 0.7403944134712219, y: 0.9699068069458008 },
		{ x: 0.746086597442627, y: 0.9693608283996582 },
		{ x: 0.7503557801246643, y: 0.9677503705024719 },
		{ x: 0.7588940858840942, y: 0.9648488163948059 },
		{ x: 0.7623500823974609, y: 0.9630200266838074 },
		{ x: 0.7656027674674988, y: 0.9621869921684265 },
		{ x: 0.773124635219574, y: 0.9596265554428101 },
		{ x: 0.77576744556427, y: 0.959517240524292 },
		{ x: 0.7836958765983582, y: 0.9577639698982239 },
		{ x: 0.7863386869430542, y: 0.9562422633171082 },
		{ x: 0.7887781858444214, y: 0.9547244310379028 },
		{ x: 0.7958934903144836, y: 0.9534826874732971 },
		{ x: 0.8005692362785339, y: 0.9512077569961548 },
		{ x: 0.8048383593559265, y: 0.949172854423523 },
		{ x: 0.8074811697006226, y: 0.947519063949585 },
		{ x: 0.8121569156646729, y: 0.9446677565574646 },
		{ x: 0.8154096603393555, y: 0.9437646865844727 },
		{ x: 0.8184590339660645, y: 0.9421951770782471 },
		{ x: 0.822524905204773, y: 0.9409302473068237 },
		{ x: 0.8259809017181396, y: 0.9389877319335938 },
		{ x: 0.8369587063789368, y: 0.934831976890564 },
		{ x: 0.855255126953125, y: 0.9264479279518127 },
		{ x: 0.8562716245651245, y: 0.9259178042411804 },
		{ x: 0.8581012487411499, y: 0.9252520799636841 },
		{ x: 0.8609473705291748, y: 0.9244706630706787 },
		{ x: 0.8621671199798584, y: 0.9239651560783386 },
		{ x: 0.8658263683319092, y: 0.9216619729995728 },
		{ x: 0.869079053401947, y: 0.9211376905441284 },
		{ x: 0.8731449246406555, y: 0.9200942516326904 },
		{ x: 0.8761943578720093, y: 0.9178023934364319 },
		{ x: 0.8776174187660217, y: 0.9169498682022095 },
		{ x: 0.8833096027374268, y: 0.9145442843437195 },
		{ x: 0.8851392269134521, y: 0.9133626818656921 },
		{ x: 0.8906281590461731, y: 0.9066638946533203 },
		{ x: 0.8932709693908691, y: 0.9048599600791931 },
		{ x: 0.8946940302848816, y: 0.903510570526123 },
		{ x: 0.8959137797355652, y: 0.9028887748718262 },
		{ x: 0.8971335887908936, y: 0.901716411113739 },
		{ x: 0.898963212966919, y: 0.9011616110801697 },
		{ x: 0.9003862738609314, y: 0.9003862738609314 },
		{ x: 0.901199460029602, y: 0.8995535969734192 },
		{ x: 0.9032323360443115, y: 0.8992106914520264 },
		{ x: 0.9040455222129822, y: 0.8989286422729492 },
		{ x: 0.9052652716636658, y: 0.8977822661399841 },
		{ x: 0.9093311429023743, y: 0.896213173866272 },
		{ x: 0.915429949760437, y: 0.8939844965934753 },
		{ x: 0.9160398244857788, y: 0.8936929702758789 },
		{ x: 0.9227485060691833, y: 0.8906986117362976 },
		{ x: 0.9231551289558411, y: 0.8889976739883423 },
		{ x: 0.9251880645751953, y: 0.8881732821464539 },
		{ x: 0.9257979393005371, y: 0.8877192735671997 },
		{ x: 0.9272210001945496, y: 0.8876994848251343 },
		{ x: 0.9288473129272461, y: 0.8864959478378296 },
		{ x: 0.929050624370575, y: 0.8858305811882019 },
		{ x: 0.9298638105392456, y: 0.8852332234382629 },
		{ x: 0.931693434715271, y: 0.884578287601471 },
		{ x: 0.9349461197853088, y: 0.8813721537590027 },
		{ x: 0.9375889301300049, y: 0.8798168897628784 },
		{ x: 0.9412482380867004, y: 0.8778915405273438 },
		{ x: 0.9418581128120422, y: 0.877295970916748 },
		{ x: 0.9432811737060547, y: 0.8771266341209412 },
		{ x: 0.947956919670105, y: 0.8742032051086426 },
		{ x: 0.9481601715087891, y: 0.8737354874610901 },
		{ x: 0.9493799805641174, y: 0.873060405254364 },
		{ x: 0.9501931071281433, y: 0.8723404407501221 },
		{ x: 0.9503964185714722, y: 0.8718761801719666 },
		{ x: 0.951006293296814, y: 0.871135950088501 },
		{ x: 0.9516161680221558, y: 0.870559811592102 },
		{ x: 0.9526326656341553, y: 0.8701949715614319 },
		{ x: 0.9544622898101807, y: 0.8696054816246033 },
		{ x: 0.956291913986206, y: 0.8682170510292053 },
		{ x: 0.9564952254295349, y: 0.8676009774208069 },
		{ x: 0.9577149748802185, y: 0.8667893409729004 },
		{ x: 0.9583248496055603, y: 0.8662256598472595 },
		{ x: 0.9593413472175598, y: 0.8650779128074646 },
		{ x: 0.9595445990562439, y: 0.8647856116294861 },
		{ x: 0.9607644081115723, y: 0.864617645740509 },
		{ x: 0.9613742828369141, y: 0.8635865449905396 },
		{ x: 0.9630005955696106, y: 0.8633132576942444 },
		{ x: 0.9632039070129395, y: 0.8628665208816528 },
		{ x: 0.9642203450202942, y: 0.8625204563140869 },
		{ x: 0.9648302793502808, y: 0.8621253371238708 },
		{ x: 0.9654401540756226, y: 0.8615747690200806 },
		{ x: 0.9662532806396484, y: 0.8608947396278381 },
		{ x: 0.9670664668083191, y: 0.8592846989631653 },
		{ x: 0.9693027138710022, y: 0.8586349487304688 },
		{ x: 0.9717422127723694, y: 0.8573991060256958 },
		{ x: 0.9739784598350525, y: 0.8552302718162537 },
		{ x: 0.9741817712783813, y: 0.8546459674835205 },
		{ x: 0.975604772567749, y: 0.8519439101219177 },
		{ x: 0.975604772567749, y: 0.8517926931381226 },
		{ x: 0.9766212701797485, y: 0.8504160046577454 },
		{ x: 0.9768245816230774, y: 0.8492400050163269 },
		{ x: 0.9774344563484192, y: 0.8481213450431824 },
		{ x: 0.9788575172424316, y: 0.8475620746612549 },
		{ x: 0.9796706438064575, y: 0.8469244241714478 },
		{ x: 0.9804838299751282, y: 0.8442149758338928 },
		{ x: 0.98109370470047, y: 0.8429694175720215 },
		{ x: 0.9817035794258118, y: 0.8405569791793823 },
		{ x: 0.9825167655944824, y: 0.839062511920929 },
		{ x: 0.9835332632064819, y: 0.8384748697280884 },
		{ x: 0.9839398264884949, y: 0.8380952477455139 },
		{ x: 0.9847530126571655, y: 0.8370485305786133 },
		{ x: 0.986176073551178, y: 0.8347960710525513 },
		{ x: 0.9869892001152039, y: 0.8341924548149109 },
		{ x: 0.9875991344451904, y: 0.8329904079437256 },
		{ x: 0.9890221357345581, y: 0.8313397169113159 },
		{ x: 0.9900386333465576, y: 0.8296422362327576 },
		{ x: 0.9912583827972412, y: 0.8285471796989441 },
		{ x: 0.9922748804092407, y: 0.82742840051651 },
		{ x: 0.9924781322479248, y: 0.8238272070884705 },
		{ x: 0.9926814436912537, y: 0.8233013153076172 },
		{ x: 0.9934946298599243, y: 0.8217588663101196 },
		{ x: 0.9947143793106079, y: 0.8198726773262024 },
		{ x: 0.9947143793106079, y: 0.8193234801292419 },
		{ x: 0.9949176907539368, y: 0.8185315132141113 },
		{ x: 0.9953242540359497, y: 0.817771852016449 },
		{ x: 0.9953242540359497, y: 0.8173622488975525 },
		{ x: 0.9955275654792786, y: 0.8168473839759827 },
		{ x: 0.9955275654792786, y: 0.8164387941360474 },
		{ x: 0.9955275654792786, y: 0.8157587647438049 },
		{ x: 0.9961374402046204, y: 0.8135480880737305 },
		{ x: 0.9965440034866333, y: 0.8126657605171204 },
		{ x: 0.9971538782119751, y: 0.8106098175048828 },
		{ x: 0.9975605010986328, y: 0.8093352913856506 },
		{ x: 0.9979670643806458, y: 0.8067378997802734 },
		{ x: 0.9983736276626587, y: 0.8054780960083008 },
		{ x: 0.9985769391059875, y: 0.8040595650672913 },
		{ x: 0.9985769391059875, y: 0.8039279580116272 },
		{ x: 0.9987802505493164, y: 0.8031715154647827 },
		{ x: 0.9987802505493164, y: 0.8023844361305237 },
		{ x: 0.9987802505493164, y: 0.8019915223121643 },
		{ x: 0.9987802505493164, y: 0.8018606305122375 },
		{ x: 0.9987802505493164, y: 0.8015989661216736 },
		{ x: 0.9987802505493164, y: 0.8015989661216736 },
		{ x: 0.9991868138313293, y: 0.8014022707939148 },
		{ x: 0.9991868138313293, y: 0.8011410236358643 },
		{ x: 0.9991868138313293, y: 0.8007494211196899 },
		{ x: 0.9991868138313293, y: 0.8006190061569214 },
		{ x: 0.9991868138313293, y: 0.800097644329071 },
		{ x: 0.9991868138313293, y: 0.7998372912406921 },
		{ x: 0.9991868138313293, y: 0.7987973093986511 },
		{ x: 0.9991868138313293, y: 0.7980191707611084 },
		{ x: 0.9993901252746582, y: 0.7974047064781189 },
		{ x: 0.9993901252746582, y: 0.7970168590545654 },
		{ x: 0.9993901252746582, y: 0.7962422966957092 },
		{ x: 0.9993901252746582, y: 0.795211911201477 },
		{ x: 0.9993901252746582, y: 0.7927753329277039 },
		{ x: 0.9995934367179871, y: 0.7901333570480347 },
		{ x: 0.9995934367179871, y: 0.7874760031700134 },
		{ x: 0.9997966885566711, y: 0.7848707437515259 },
		{ x: 1, y: 0.7810416221618652 },
		{ x: 1, y: 0.7786924242973328 },
		{ x: 1, y: 0.7744017839431763 },
		{ x: 1, y: 0.7665575742721558 },
		{ x: 1, y: 0.7594565153121948 },
		{ x: 1, y: 0.7556067705154419 },
		{ x: 1, y: 0.7552587389945984 },
		{ x: 1, y: 0.7552587389945984 },
		{ x: 1, y: 0.7552587389945984 },
		{ x: 1, y: 0.7552587389945984 },
		{ x: 1, y: 0.7552587389945984 },
		{ x: 1, y: 0.7552587389945984 },
		{ x: 1, y: 0.7552587389945984 },
		{ x: 1, y: 0.7552587389945984 },
		{ x: 1, y: 0.7552587389945984 },
	],
	precisionUnweighted: 0.8301436901092529,
	precisionWeighted: 0.8510709404945374,
	recallUnweigted: 0.7584391832351685,
	recallWeighted: 0.8567480444908142,
	rocData: [
		{ x: 0, y: 0 },
		{ x: 0, y: 0 },
		{ x: 0, y: 0 },
		{ x: 0, y: 0.001829640170766416 },
		{ x: 0.0006273525721455458, y: 0.10225655621061192 },
		{ x: 0.0018820577164366374, y: 0.1868265907704818 },
		{ x: 0.0018820577164366374, y: 0.2427322626550112 },
		{ x: 0.003136762860727729, y: 0.2933523073795487 },
		{ x: 0.006273525721455458, y: 0.36043911364098397 },
		{ x: 0.00878293601003764, y: 0.40882293149014026 },
		{ x: 0.01066499372647428, y: 0.4336247204716406 },
		{ x: 0.012547051442910916, y: 0.4669648302500508 },
		{ x: 0.014429109159347553, y: 0.5051839804838382 },
		{ x: 0.016938519447929738, y: 0.5222606220776581 },
		{ x: 0.018820577164366373, y: 0.5364911567391746 },
		{ x: 0.02132998745294856, y: 0.5495019312868469 },
		{ x: 0.024466750313676285, y: 0.5696279731652775 },
		{ x: 0.026976160602258468, y: 0.5840618011791014 },
		{ x: 0.028858218318695106, y: 0.5917869485667818 },
		{ x: 0.031994981179422836, y: 0.6015450294775361 },
		{ x: 0.03450439146800502, y: 0.6147590973775158 },
		{ x: 0.03513174404015056, y: 0.6243138849359626 },
		{ x: 0.037641154328732745, y: 0.6338686724944095 },
		{ x: 0.04203262233375157, y: 0.6476926204513113 },
		{ x: 0.045796737766624844, y: 0.6568408213051433 },
		{ x: 0.04956085319949812, y: 0.6633462085789794 },
		{ x: 0.05081555834378921, y: 0.6694450091482008 },
		{ x: 0.052070263488080304, y: 0.6753405163651148 },
		{ x: 0.05269761606022585, y: 0.6824557836958731 },
		{ x: 0.05520702634880803, y: 0.6916039845497052 },
		{ x: 0.055834378920953574, y: 0.6997357186420005 },
		{ x: 0.05834378920953576, y: 0.7031917056312259 },
		{ x: 0.06398996235884567, y: 0.7223012807481195 },
		{ x: 0.06587202007528231, y: 0.7294165480788778 },
		{ x: 0.07089084065244668, y: 0.7403943891034763 },
		{ x: 0.07277289836888332, y: 0.7460866029680829 },
		{ x: 0.07716436637390213, y: 0.7503557633665379 },
		{ x: 0.08531994981179424, y: 0.7588940841634478 },
		{ x: 0.0903387703889586, y: 0.7623500711526733 },
		{ x: 0.09284818067754078, y: 0.7656027647895913 },
		{ x: 0.10037641154328733, y: 0.7731246188249644 },
		{ x: 0.10100376411543287, y: 0.7757674324049604 },
		{ x: 0.10664993726474278, y: 0.7836958731449482 },
		{ x: 0.11104140526976161, y: 0.7863386867249441 },
		{ x: 0.11543287327478043, y: 0.7887782069526327 },
		{ x: 0.11982434127979925, y: 0.795893474283391 },
		{ x: 0.12672521957340024, y: 0.8005692213864607 },
		{ x: 0.1329987452948557, y: 0.8048383817849156 },
		{ x: 0.13801756587202008, y: 0.8074811953649116 },
		{ x: 0.1468005018820577, y: 0.8121569424679813 },
		{ x: 0.14993726474278546, y: 0.8154096361048994 },
		{ x: 0.1549560853199498, y: 0.81845903638951 },
		{ x: 0.15934755332496864, y: 0.8225249034356577 },
		{ x: 0.1656210790464241, y: 0.8259808904248831 },
		{ x: 0.18005018820577165, y: 0.8369587314494816 },
		{ x: 0.2095357590966123, y: 0.8552551331571457 },
		{ x: 0.21141781681304894, y: 0.8562715999186826 },
		{ x: 0.21392722710163112, y: 0.8581012400894491 },
		{ x: 0.21706398996235884, y: 0.8609473470217524 },
		{ x: 0.2189460476787955, y: 0.8621671071355966 },
		{ x: 0.22710163111668757, y: 0.8658263874771295 },
		{ x: 0.22961104140526975, y: 0.8690790811140475 },
		{ x: 0.23400250941028858, y: 0.8731449481601952 },
		{ x: 0.24215809284818068, y: 0.8761943484448058 },
		{ x: 0.2452948557089084, y: 0.8776174019109575 },
		{ x: 0.2547051442910916, y: 0.8833096157755641 },
		{ x: 0.2590966122961104, y: 0.8851392559463306 },
		{ x: 0.28293601003764113, y: 0.8906281764586298 },
		{ x: 0.28983688833124216, y: 0.8932709900386258 },
		{ x: 0.2948557089084065, y: 0.8946940435047774 },
		{ x: 0.2973651191969887, y: 0.8959138036186217 },
		{ x: 0.30175658720200754, y: 0.897133563732466 },
		{ x: 0.3042659974905897, y: 0.8989632039032324 },
		{ x: 0.3074027603513174, y: 0.900386257369384 },
		{ x: 0.31053952321204514, y: 0.9011994307786135 },
		{ x: 0.3124215809284818, y: 0.9032323643016873 },
		{ x: 0.3136762860727729, y: 0.9040455377109169 },
		{ x: 0.3180677540777917, y: 0.9052652978247612 },
		{ x: 0.32496863237139273, y: 0.9093311648709087 },
		{ x: 0.33500627352572143, y: 0.9154299654401301 },
		{ x: 0.33626097867001253, y: 0.9160398454970522 },
		{ x: 0.349435382685069, y: 0.9227485261231958 },
		{ x: 0.35570890840652447, y: 0.9231551128278105 },
		{ x: 0.3594730238393977, y: 0.9251880463508844 },
		{ x: 0.3613550815558344, y: 0.9257979264078064 },
		{ x: 0.3619824341279799, y: 0.9272209798739581 },
		{ x: 0.3670012547051443, y: 0.9288473266924172 },
		{ x: 0.3695106649937265, y: 0.9290506200447245 },
		{ x: 0.3720200752823087, y: 0.9298637934539541 },
		{ x: 0.3751568381430364, y: 0.9316934336247205 },
		{ x: 0.38833124215809284, y: 0.9349461272616385 },
		{ x: 0.39523212045169387, y: 0.9375889408416345 },
		{ x: 0.40401505646173147, y: 0.9412482211831673 },
		{ x: 0.4065244667503137, y: 0.9418581012400894 },
		{ x: 0.4077791718946048, y: 0.9432811547062411 },
		{ x: 0.4209535759096612, y: 0.9479569018093108 },
		{ x: 0.4228356336260979, y: 0.9481601951616182 },
		{ x: 0.4259723964868256, y: 0.9493799552754625 },
		{ x: 0.42910915934755334, y: 0.950193128684692 },
		{ x: 0.43099121706398996, y: 0.9503964220369994 },
		{ x: 0.4341279799247177, y: 0.9510063020939216 },
		{ x: 0.4366373902132999, y: 0.9516161821508436 },
		{ x: 0.4385194479297365, y: 0.9526326489123805 },
		{ x: 0.44165621079046424, y: 0.954462289083147 },
		{ x: 0.4479297365119197, y: 0.9562919292539134 },
		{ x: 0.4504391468005019, y: 0.9564952226062208 },
		{ x: 0.45420326223337515, y: 0.9577149827200651 },
		{ x: 0.45671267252195735, y: 0.9583248627769871 },
		{ x: 0.4617314930991217, y: 0.9593413295385241 },
		{ x: 0.4629861982434128, y: 0.9595446228908314 },
		{ x: 0.4642409033877039, y: 0.9607643830046757 },
		{ x: 0.46863237139272274, y: 0.9613742630615979 },
		{ x: 0.47051442910915936, y: 0.9630006098800569 },
		{ x: 0.472396486825596, y: 0.9632039032323643 },
		{ x: 0.4742785445420326, y: 0.9642203699939011 },
		{ x: 0.47616060225846923, y: 0.9648302500508233 },
		{ x: 0.47867001254705144, y: 0.9654401301077454 },
		{ x: 0.48180677540777916, y: 0.966253303516975 },
		{ x: 0.4887076537013802, y: 0.9670664769262045 },
		{ x: 0.49247176913425345, y: 0.9693027038015857 },
		{ x: 0.4987452948557089, y: 0.9717422240292742 },
		{ x: 0.5087829360100377, y: 0.9739784509046554 },
		{ x: 0.5112923462986199, y: 0.9741817442569628 },
		{ x: 0.5232120451693852, y: 0.9756047977231145 },
		{ x: 0.5238393977415308, y: 0.9756047977231145 },
		{ x: 0.5301129234629862, y: 0.9766212644846514 },
		{ x: 0.5351317440401505, y: 0.9768245578369588 },
		{ x: 0.5401505646173149, y: 0.9774344378938808 },
		{ x: 0.5432873274780426, y: 0.9788574913600325 },
		{ x: 0.5464240903387704, y: 0.979670664769262 },
		{ x: 0.5583437892095358, y: 0.9804838381784916 },
		{ x: 0.5639899623588457, y: 0.9810937182354137 },
		{ x: 0.5746549560853199, y: 0.9817035982923359 },
		{ x: 0.581555834378921, y: 0.9825167717015654 },
		{ x: 0.5846925972396487, y: 0.9835332384631023 },
		{ x: 0.5865746549560853, y: 0.983939825167717 },
		{ x: 0.5915934755332497, y: 0.9847529985769465 },
		{ x: 0.6022584692597239, y: 0.9861760520430982 },
		{ x: 0.6053952321204517, y: 0.9869892254523277 },
		{ x: 0.6110414052697616, y: 0.9875991055092499 },
		{ x: 0.6191969887076537, y: 0.9890221589754015 },
		{ x: 0.6273525721455459, y: 0.9900386257369383 },
		{ x: 0.6329987452948557, y: 0.9912583858507826 },
		{ x: 0.6386449184441656, y: 0.9922748526123196 },
		{ x: 0.6549560853199499, y: 0.9924781459646269 },
		{ x: 0.657465495608532, y: 0.9926814393169343 },
		{ x: 0.6649937264742786, y: 0.9934946127261639 },
		{ x: 0.6744040150564617, y: 0.9947143728400081 },
		{ x: 0.676913425345044, y: 0.9947143728400081 },
		{ x: 0.6806775407779172, y: 0.9949176661923155 },
		{ x: 0.6844416562107905, y: 0.9953242528969303 },
		{ x: 0.6863237139272271, y: 0.9953242528969303 },
		{ x: 0.6888331242158093, y: 0.9955275462492377 },
		{ x: 0.6907151819322459, y: 0.9955275462492377 },
		{ x: 0.6938519447929736, y: 0.9955275462492377 },
		{ x: 0.704516938519448, y: 0.9961374263061598 },
		{ x: 0.7089084065244667, y: 0.9965440130107746 },
		{ x: 0.7189460476787954, y: 0.9971538930676966 },
		{ x: 0.7252195734002509, y: 0.9975604797723114 },
		{ x: 0.7377666248431619, y: 0.9979670664769262 },
		{ x: 0.7440401505646174, y: 0.9983736531815409 },
		{ x: 0.7509410288582183, y: 0.9985769465338483 },
		{ x: 0.7515683814303639, y: 0.9985769465338483 },
		{ x: 0.7553324968632371, y: 0.9987802398861557 },
		{ x: 0.7590966122961104, y: 0.9987802398861557 },
		{ x: 0.760978670012547, y: 0.9987802398861557 },
		{ x: 0.7616060225846926, y: 0.9987802398861557 },
		{ x: 0.7628607277289837, y: 0.9987802398861557 },
		{ x: 0.7628607277289837, y: 0.9987802398861557 },
		{ x: 0.7641154328732748, y: 0.9991868265907705 },
		{ x: 0.7653701380175659, y: 0.9991868265907705 },
		{ x: 0.7672521957340025, y: 0.9991868265907705 },
		{ x: 0.767879548306148, y: 0.9991868265907705 },
		{ x: 0.7703889585947302, y: 0.9991868265907705 },
		{ x: 0.7716436637390214, y: 0.9991868265907705 },
		{ x: 0.7766624843161857, y: 0.9991868265907705 },
		{ x: 0.7804265997490589, y: 0.9991868265907705 },
		{ x: 0.7835633626097867, y: 0.9993901199430779 },
		{ x: 0.7854454203262233, y: 0.9993901199430779 },
		{ x: 0.7892095357590966, y: 0.9993901199430779 },
		{ x: 0.794228356336261, y: 0.9993901199430779 },
		{ x: 0.8061480552070264, y: 0.9993901199430779 },
		{ x: 0.8193224592220828, y: 0.9995934132953852 },
		{ x: 0.8324968632371392, y: 0.9995934132953852 },
		{ x: 0.8456712672521958, y: 0.9997967066476926 },
		{ x: 0.8651191969887077, y: 1 },
		{ x: 0.877038895859473, y: 1 },
		{ x: 0.8989962358845671, y: 1 },
		{ x: 0.9397741530740276, y: 1 },
		{ x: 0.9774153074027604, y: 1 },
		{ x: 0.9981179422835633, y: 1 },
		{ x: 1, y: 1 },
		{ x: 1, y: 1 },
		{ x: 1, y: 1 },
		{ x: 1, y: 1 },
		{ x: 1, y: 1 },
		{ x: 1, y: 1 },
		{ x: 1, y: 1 },
		{ x: 1, y: 1 },
		{ x: 1, y: 1 },
	],
}
