use crate::components::progress::ScrobbleProgressBar;
use musicbirb::Track;
use ratatui::{prelude::*, widgets::*};

pub fn render_now_playing(
	f: &mut Frame,
	area: Rect,
	current_track: Option<&Track>,
	time: f64,
	paused: bool,
	scrobble_mark_pos: Option<f64>,
) {
	let (title, artist, album, ratio, dur, scrobble_ratio) = if let Some(t) = current_track {
		let duration = t.duration_secs as f64;
		let r = if duration > 0.0 {
			(time / duration).clamp(0.0, 1.0)
		} else {
			0.0
		};
		let s_r = scrobble_mark_pos.and_then(|m| {
			if m <= duration {
				Some(m / duration)
			} else {
				None
			}
		});
		(
			t.title.clone(),
			t.artist.clone(),
			t.album.clone(),
			r,
			format!("{:.0}s / {}s", time, t.duration_secs),
			s_r,
		)
	} else {
		(
			"Idle".into(),
			"".into(),
			"".into(),
			0.0,
			"--/--".into(),
			None,
		)
	};

	let bar = Block::bordered().title(" Now Playing ").inner(area);
	f.render_widget(Block::bordered().title(" Now Playing "), area);
	let b_lay = Layout::vertical([
		Constraint::Length(1),
		Constraint::Length(1),
		Constraint::Length(1),
		Constraint::Length(1),
	])
	.split(bar);

	f.render_widget(
		Paragraph::new(format!(
			"{}{}",
			title,
			if paused { " [PAUSED]" } else { "" }
		))
		.cyan()
		.bold(),
		b_lay[0],
	);
	f.render_widget(Paragraph::new(format!("{} - {}", artist, album)), b_lay[1]);

	let progress = ScrobbleProgressBar::new(ratio, scrobble_ratio);
	f.render_widget(progress, b_lay[2]);

	f.render_widget(Paragraph::new(dur).alignment(Alignment::Right), b_lay[3]);
}
