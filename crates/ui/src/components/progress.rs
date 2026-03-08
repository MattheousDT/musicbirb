use ratatui::{
	buffer::Buffer,
	layout::Rect,
	style::{Color, Style},
	widgets::Widget,
};

pub struct ScrobbleProgressBar {
	ratio: f64,
	scrobble_ratio: Option<f64>,
}

impl ScrobbleProgressBar {
	pub fn new(ratio: f64, scrobble_ratio: Option<f64>) -> Self {
		Self {
			ratio: ratio.clamp(0.0, 1.0),
			scrobble_ratio: scrobble_ratio.map(|r| r.clamp(0.0, 1.0)),
		}
	}
}

impl Widget for ScrobbleProgressBar {
	fn render(self, area: Rect, buf: &mut Buffer) {
		if area.width == 0 || area.height == 0 {
			return;
		}

		let width = area.width as usize;
		let filled = (width as f64 * self.ratio).round() as usize;
		let scrobble_idx = self
			.scrobble_ratio
			.map(|r| (width as f64 * r).round() as usize)
			.filter(|&idx| idx < width);

		let track_style = Style::default().fg(Color::DarkGray);
		let filled_style = Style::default().fg(Color::Green);
		let mark_style = Style::default().fg(Color::Yellow);

		for x in 0..width {
			let pos_x = area.x + x as u16;
			let is_mark = scrobble_idx == Some(x);
			let is_filled = x < filled;

			let style = if is_mark {
				mark_style
			} else if is_filled {
				filled_style
			} else {
				track_style
			};

			buf.set_string(pos_x, area.y, "█", style);
		}
	}
}
