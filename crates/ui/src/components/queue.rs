use musicbirb::Track;
use ratatui::{prelude::*, widgets::*};

pub fn render_queue(f: &mut Frame, area: Rect, queue: &[Track], pos: usize) {
	let pad_width = queue.len().to_string().len().max(1);

	let items: Vec<ListItem> = queue
		.iter()
		.enumerate()
		.map(|(i, t)| {
			let style = if i == pos {
				Style::default().fg(Color::Yellow).bold()
			} else {
				Style::default()
			};
			ListItem::new(format!(
				"{:0width$}. {} - {}",
				i + 1,
				t.title,
				t.artist,
				width = pad_width
			))
			.style(style)
		})
		.collect();

	let mut l_state = ListState::default().with_selected(Some(pos));
	f.render_stateful_widget(
		List::new(items).block(Block::bordered().title(" Queue ")),
		area,
		&mut l_state,
	);
}
