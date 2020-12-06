use html::{component, html, style};
use rand::prelude::*;
// use wasm_bindgen::prelude::*;
// use wasm_bindgen::JsCast;

#[derive(Clone)]
pub enum Language {
	Go,
	Javascript,
	Python,
	Ruby,
}

#[component]
pub fn Code(code: String, hide_line_numbers: Option<bool>, _language: Language) {
	let hide_line_numbers = hide_line_numbers.unwrap_or(false);
	html! {
		<div class="code">
			<div class="code-inner-wrapper">
			{if !hide_line_numbers {
				Some(html! {
					<LineNumbers count={count_lines(code.clone())} />
				})
			} else {
				None
			}}
				<div
					class="code-inner-code"
					dangerously_set_inner_html={format!("{{'__html': {}}}", code)}
				/>
			</div>
		</div>
	}
}

#[derive(Clone)]
pub struct CodeText {
	go: String,
	javascript: String,
	python: String,
	ruby: String,
}

#[component]
pub fn CodeSelect(hide_line_numbers: Option<bool>, code_text: CodeText) {
	// TODO this sucks
	let option_id = rand::thread_rng().gen::<f32>().to_string();
	html! {
		<div class="code code-grid">
			<CodeOption
				checked={true}
				hide_line_numbers={hide_line_numbers}
				code={code_text.go}
				id={option_id.clone()}
				language={Language::Go}
			/>
			<CodeOption
				checked={false}
				hide_line_numbers={hide_line_numbers}
				code={code_text.javascript}
				id={option_id.clone()}
				language={Language::Javascript}
			/>
			<CodeOption
				checked={false}
				hide_line_numbers={hide_line_numbers}
				code={code_text.python}
				id={option_id.clone()}
				language={Language::Python}
			/>
			<CodeOption
				checked={false}
				hide_line_numbers={hide_line_numbers}
				code={code_text.ruby}
				id={option_id}
				language={Language::Ruby}
			/>
		</div>
	}
}

#[component]
pub fn CodeOption(
	checked: Option<bool>,
	code: String,
	hide_line_numbers: Option<bool>,
	id: String,
	language: Language,
) {
	let name_for_language = match language {
		Language::Go => "go".to_owned(),
		Language::Javascript => "javascript".to_owned(),
		Language::Python => "python".to_owned(),
		Language::Ruby => "ruby".to_owned(),
	};
	// TODO this sucks
	let option_id = rand::thread_rng().gen::<f32>().to_string();
	let hide_line_numbers = hide_line_numbers.unwrap_or(false);
	let style = style! {
		"grid-area" => name_for_language,
	};
	html! {
		<>
		<input
			checked={checked}
			class={"code-radio-input".to_owned()}
			data-lang={name_for_language.clone()}
			id={option_id.clone()}
			name={id}
			type={"radio".to_owned()}
			value={name_for_language.clone()}
		/>
		<label
			class={"code-radio-label".to_owned()}
			data-lang={name_for_language.clone()}
			for={option_id}
			style={style}
		>
			{name_for_language.clone()}
		</label>
		<div class="code-inner-wrapper" data-lang={name_for_language}>
			{if !hide_line_numbers {
				Some(html! {
					<LineNumbers count={count_lines(code.clone())} />
				})
			} else {
				None
			}}
				<div
					class="code-inner-code"
					dangerously_set_inner_html={format!("{{'__html': {}}}", code)}
				/>
			</div>
		</>
	}
}

#[component]
pub fn LineNumbers(count: usize) {
	html! {
		<div class="code-line-numbers-wrapper">
			{(0..count).map(|index| html! {
				<div class="code-line-numbers">
					{(index + 1).to_string()}
				</div>
			}).collect::<Vec<_>>()}
		</div>
	}
}

#[component]
pub fn InlineCode() {
	html! {
		<span class="inline-code-wrapper">
			{children}
		</span>
	}
}

fn count_lines(text: String) -> usize {
	let n_lines = text.split('\n').count();
	if text.ends_with('\n') {
		n_lines - 1
	} else {
		n_lines
	}
}

// pub fn boot_code_select() {
// 	let document = web_sys::window().unwrap().document().unwrap();
// 	let radio_elements = document.query_selector_all("input[type=radio]").unwrap();
// 	let callback_fn = Closure::<dyn Fn(_)>::wrap(Box::new(move |event: web_sys::Event| {
// 		let document = web_sys::window().unwrap().document().unwrap();
// 		let lang = event
// 			.current_target()
// 			.unwrap()
// 			.dyn_into::<web_sys::HtmlElement>()
// 			.unwrap()
// 			.dataset()
// 			.get("lang")
// 			.unwrap();
// 		let lang_elements = document
// 			.query_selector_all(&format!("input[type=radio][data-lang={}]", lang))
// 			.unwrap();
// 		for index in 0..lang_elements.length() {
// 			let lang_element = lang_elements
// 				.get(index)
// 				.unwrap()
// 				.dyn_into::<web_sys::HtmlInputElement>()
// 				.unwrap();
// 			lang_element.set_checked(true);
// 		}
// 	}));
// 	for index in 0..radio_elements.length() {
// 		let radio_element = radio_elements
// 			.get(index)
// 			.unwrap()
// 			.dyn_into::<web_sys::HtmlInputElement>()
// 			.unwrap();
// 		radio_element
// 			.add_event_listener_with_callback("click", callback_fn.as_ref().unchecked_ref())
// 			.unwrap();
// 	}
// }
