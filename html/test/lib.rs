use html::{component, html};

#[component]
fn Test() {
	return html!(
	  <div>{"Hello World"}</div>
	);
}

#[test]
fn test() {
	let html = html!(<Test />).render_to_string();
	assert_eq!(html, "<div>Hello World</div>");
}
