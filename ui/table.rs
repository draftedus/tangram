use html::{classes, component, html, style};

#[component]
pub fn Table(width: Option<String>) {
	let style = style! {
		"width" => width.unwrap_or_else(|| "auto".into()),
	};
	html! {
		<div class="table-wrapper">
			<table class="table" style={style}>
				{children}
			</table>
		</div>
	}
}

#[component]
pub fn TableHeader() {
	html! {
		<thead class="table-header">
			{children}
		</thead>
	}
}

#[component]
pub fn TableBody() {
	html! {
		<tbody>{children}</tbody>
	}
}

#[component]
pub fn TableRow(color: Option<String>) {
	let style = style! {
		"background-color" => color,
	};
	html! {
		<tr style={style}>
			{children}
		</tr>
	}
}

#[derive(Clone)]
pub enum TextAlign {
	Left,
	Center,
	Right,
}

#[component]
pub fn TableHeaderCell(color: Option<String>, expand: Option<bool>, text_align: Option<TextAlign>) {
	let style = style! {
		"background-color" => color,
	};
	let text_align_class = text_align
		.map(|text_align| match text_align {
			TextAlign::Left => "table-align-left",
			TextAlign::Right => "table-align-right",
			TextAlign::Center => "table-align-center",
		})
		.unwrap_or("table-align-left");
	let expand = expand.and_then(|expand| if expand { Some("table-expand") } else { None });
	let class = classes!("table-header-cell", text_align_class, expand);
	html! {
		<th class={class} style={style}>
			{children}
		</th>
	}
}

#[component]
pub fn TableCell(color: Option<String>, expand: Option<bool>) {
	let style = style! {
		"background-color" => color,
	};
	let expand = expand.and_then(|expand| if expand { Some("table-expand") } else { None });
	let class = classes!("table-cell", expand);
	html! {
		<td class={class} style={style}>
			{children}
		</td>
	}
}
