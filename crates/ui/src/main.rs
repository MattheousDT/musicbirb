mod components;

use anyhow::Result;
use components::{now_playing::render_now_playing, queue::render_queue};
use crossterm::{
	ExecutableCommand,
	event::{self, Event, KeyCode},
	terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use musicbirb::mpv::MpvBackend;
use musicbirb::{AlbumId, Musicbirb, PlaylistId, SubsonicClient, TrackId};
use ratatui::{prelude::*, widgets::*};
use ratatui_image::{StatefulImage, picker::Picker, protocol::StatefulProtocol};
use std::{
	env,
	io::stdout,
	sync::{Arc, Mutex},
	time::Duration,
};

#[tokio::main]
async fn main() -> Result<()> {
	dotenvy::dotenv().ok();
	let url = env::var("SUBSONIC_URL")?;
	let user = env::var("SUBSONIC_USER")?;
	let pass = env::var("SUBSONIC_PASS")?;

	let api = SubsonicClient::new(&url, &user, &pass)?;
	let player = Arc::new(MpvBackend::new()?);
	let core = Musicbirb::new(api, player);
	let mut state_rx = core.subscribe();

	enable_raw_mode()?;
	stdout().execute(EnterAlternateScreen)?;
	let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

	let picker = Picker::from_query_stdio().unwrap_or_else(|_| Picker::halfblocks());
	let mut input = String::new();
	let mut input_mode = false;
	let error_msg = Arc::new(Mutex::new(None::<String>));
	let info_msg = Arc::new(Mutex::new(None::<String>));

	let mut last_art_arc = None;
	let image_protocol = Arc::new(Mutex::new(None::<StatefulProtocol>));

	loop {
		let state = state_rx.borrow().clone();

		let current_track = state.queue.get(state.queue_position).cloned();
		let queue = state.queue;
		let pos = state.queue_position;
		let current_art = state.current_art;

		if let Some(art) = current_art {
			let is_new = match &last_art_arc {
				Some(last) => !Arc::ptr_eq(&art, last),
				None => true,
			};

			if is_new {
				last_art_arc = Some(Arc::clone(&art));
				*image_protocol.lock().unwrap() = None;

				let p_clone = Arc::clone(&image_protocol);
				let pick_clone = picker.clone();
				tokio::spawn(async move {
					let prot = pick_clone.new_resize_protocol((*art).clone());
					*p_clone.lock().unwrap() = Some(prot);
				});
			}
		} else if last_art_arc.is_some() {
			last_art_arc = None;
			*image_protocol.lock().unwrap() = None;
		}

		let current_err = error_msg.lock().unwrap().clone();
		let current_info = info_msg.lock().unwrap().clone();
		let mut prot_lock = image_protocol.lock().unwrap();

		terminal.draw(|f| {
			let main = Layout::vertical([
				Constraint::Min(0),
				Constraint::Length(6),
				Constraint::Length(3),
			])
			.split(f.area());
			let top = Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
				.split(main[0]);

			let art_area = Block::bordered().title(" Album Art ").inner(top[0]);
			f.render_widget(Block::bordered().title(" Album Art "), top[0]);

			if let Some(p) = prot_lock.as_mut() {
				f.render_stateful_widget(StatefulImage::default(), art_area, p);
			} else {
				let txt = if last_art_arc.is_some() {
					"Loading..."
				} else {
					"No Art"
				};
				f.render_widget(Paragraph::new(txt).alignment(Alignment::Center), art_area);
			}

			render_queue(f, top[1], &queue, pos);
			render_now_playing(
				f,
				main[1],
				current_track.as_ref(),
				&state.sync,
				state.scrobble_mark_pos,
			);

			if let Some(e) = current_err {
				f.render_widget(
					Paragraph::new(e)
						.red()
						.block(Block::bordered().title(" Error ")),
					main[2],
				);
			} else if let Some(i) = current_info {
				f.render_widget(
					Paragraph::new(i)
						.blue()
						.block(Block::bordered().title(" Info ")),
					main[2],
				);
			} else {
				let (style, title) = if input_mode {
					(Style::default().fg(Color::Yellow), " Input (Enter/Esc) ")
				} else {
					(Style::default(), " Normal (i to type) ")
				};
				f.render_widget(
					Paragraph::new(format!("> {}", input))
						.style(style)
						.block(Block::bordered().title(title)),
					main[2],
				);
			}
		})?;

		if event::poll(Duration::from_millis(16))? {
			if let Event::Key(key) = event::read()? {
				if key.code != KeyCode::Null {
					*error_msg.lock().unwrap() = None;
					if !input_mode {
						*info_msg.lock().unwrap() = None;
					}
				}

				if input_mode {
					match key.code {
						KeyCode::Esc => input_mode = false,
						KeyCode::Backspace => {
							input.pop();
						}
						KeyCode::Char(c) => {
							input.push(c);
						}
						KeyCode::Enter => {
							let raw = input.clone();
							input.clear();
							input_mode = false;
							if !raw.trim().is_empty() {
								let c = Arc::clone(&core);
								let i_m = Arc::clone(&info_msg);
								let e_m = Arc::clone(&error_msg);
								tokio::spawn(async move {
									*i_m.lock().unwrap() = Some("Fetching...".into());
									let res = if raw.starts_with("al:") {
										c.queue_album(&AlbumId(
											raw.strip_prefix("al:").unwrap().to_string(),
										))
										.await
										.map(|count| format!("Added {} tracks", count))
									} else if raw.starts_with("pl:") {
										c.queue_playlist(&PlaylistId(
											raw.strip_prefix("pl:").unwrap().to_string(),
										))
										.await
										.map(|count| format!("Added {} tracks", count))
									} else {
										c.queue_track(&TrackId(raw.clone()))
											.await
											.map(|_| "Added track".into())
									};
									match res {
										Ok(m) => *i_m.lock().unwrap() = Some(m),
										Err(e) => {
											*i_m.lock().unwrap() = None;
											*e_m.lock().unwrap() = Some(e.to_string());
										}
									}
								});
							}
						}
						_ => {}
					}
				} else {
					match key.code {
						KeyCode::Esc => break,
						KeyCode::Char('i') | KeyCode::Char('/') => input_mode = true,
						KeyCode::Char(' ') => {
							let _ = core.toggle_pause();
						}
						KeyCode::Char('n') => {
							let _ = core.next();
						}
						KeyCode::Char('p') => {
							let _ = core.prev();
						}
						KeyCode::Left => {
							let _ = core.seek(-5.0);
						}
						KeyCode::Right => {
							let _ = core.seek(5.0);
						}
						_ => {}
					}
				}
			}
		}
	}
	disable_raw_mode()?;
	stdout().execute(LeaveAlternateScreen)?;
	Ok(())
}
