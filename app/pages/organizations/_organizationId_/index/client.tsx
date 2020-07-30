export {}
// ui.toaster.show({
// 	backgroundColor: ui.colors.blue,
// 	message: `🎉 You are now on the ${plan} plan!`,
// })

// 	let onAddPaymentMethod = async () => {
// 		try {
// 			ui.progress.start()
// 			let response = await fetch(`/start-stripe-checkout`, {
// 				method: 'POST',
// 			})
// 			if (!response.ok) {
// 				throw Error()
// 			}
// 			let { sessionId } = await response.json()
// 			let stripe = await loadStripe(props.stripePublishableKey)
// 			await stripe.redirectToCheckout({ sessionId })
// 		} finally {
// 			ui.progress.stop()
// 		}
// 	}

// 	useEffect(() => {
// 		if (props.stripeCheckoutSessionId) {
// 			finishStripeCheckout(props.id, props.stripeCheckoutSessionId)
// 		}
// 	})

// async function loadStripe(stripeKey: string): Promise<any> {
// 	let stripeUrl = 'https://js.stripe.com/v3'
// 	let existingTag = document.querySelector(`script[src="${stripeUrl}"]`)
// 	if (!existingTag) {
// 		let scriptTag = document.createElement('script')
// 		scriptTag.src = stripeUrl
// 		document.head.appendChild(scriptTag)
// 		await new Promise((resolve, reject) => {
// 			scriptTag.addEventListener('load', () => {
// 				if (Stripe) {
// 					resolve(Stripe)
// 				} else {
// 					reject(new Error('stripe.js not available'))
// 				}
// 			})
// 			scriptTag.addEventListener('error', () => {
// 				reject(new Error('stripe.js failed to load '))
// 			})
// 		})
// 	}
// 	return Stripe(stripeKey)
// }

// async function finishStripeCheckout(
// 	organizationId: string,
// 	stripeCheckoutSessionId: string,
// ) {
// 	try {
// 		ui.progress.start()
// 		let response = await fetch(
// 			`/api/organizations/${organizationId}/billing/finish-stripe-checkout`,
// 			{
// 				body: JSON.stringify({ stripeCheckoutSessionId }),
// 				method: 'POST',
// 			},
// 		)
// 		if (!response.ok) throw Error()
// 		await response.text()
// 	} catch {
// 		ui.toaster.showError('an error occurred')
// 	} finally {
// 		ui.progress.stop()
// 		location.reload()
// 	}
// }
