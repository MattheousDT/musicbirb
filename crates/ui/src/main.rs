mod components;

use anyhow::Result;
use components::{now_playing::render_now_playing, queue::render_queue};
use crossterm::{
	ExecutableCommand,
	event::{self, Event, KeyCode},
	terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use musicbirb::mpv::MpvBackend;
use musicbirb::{AlbumId, AppSettings, Musicbirb, PlaylistId, TrackId};
use musicbirb::{AuthCredential, AuthStep, Authenticator};
use ratatui::{prelude::*, widgets::*};
use ratatui_image::{StatefulImage, picker::Picker, protocol::StatefulProtocol};
use std::{
	io::stdout,
	sync::{Arc, Mutex},
	time::Duration,
};

#[derive(PartialEq)]
enum UiMode {
	Login,
	Main,
}

enum AppEvent {
	LoginSuccess(musicbirb::settings::AccountConfig, Arc<dyn musicbirb::Provider>),
	LoginFailed(String),
}

#[tokio::main]
async fn main() -> Result<()> {
	let mut settings = AppSettings::load(None);
	let mut mode = UiMode::Login;
	let mut login_state = components::login::LoginState::default();
	if settings.accounts.is_empty() {
		login_state.focus = components::login::LoginFocus::Url;
	}

	let player = Arc::new(MpvBackend::new()?);
	let core = Musicbirb::new(None, player);
	let state_rx = core.subscribe();

	let (app_tx, mut app_rx) = tokio::sync::mpsc::unbounded_channel::<AppEvent>();
	let error_msg = Arc::new(Mutex::new(None::<String>));
	let info_msg = Arc::new(Mutex::new(None::<String>));

	// Initial load auto-connect logic
	// Only auto-connect if there is exactly one account saved.
	// If there are 0 or >1, we stay in Login mode to let the user choose or add.
	if settings.accounts.len() == 1 {
		if let Some(acc) = settings.accounts.first() {
			*info_msg.lock().unwrap() = Some(format!("Auto-connecting to {}...", acc.username));
			let acc_c = acc.clone();
			let tx_c = app_tx.clone();
			tokio::spawn(async move {
				let entry = match keyring::Entry::new("musicbirb_subsonic", &acc_c.id) {
					Ok(e) => e,
					Err(e) => {
						let _ = tx_c.send(AppEvent::LoginFailed(format!("Keyring init error: {}", e)));
						return;
					}
				};
				let cred_str = match entry.get_password() {
					Ok(p) => p,
					Err(e) => {
						let _ = tx_c.send(AppEvent::LoginFailed(format!("Keychain retrieval failed: {}", e)));
						return;
					}
				};
				let credential = match serde_json::from_str::<AuthCredential>(&cred_str) {
					Ok(c) => c,
					Err(_) => AuthCredential::Password(cred_str.to_string()),
				};

				let authenticator = Authenticator::new();
				match authenticator
					.connect_with_credential(
						acc_c.provider.clone(),
						acc_c.url.clone(),
						acc_c.username.clone(),
						credential,
					)
					.await
				{
					Ok(provider) => {
						let _ = tx_c.send(AppEvent::LoginSuccess(acc_c, provider));
					}
					Err(e) => {
						let _ = tx_c.send(AppEvent::LoginFailed(format!("Provider error: {}", e)));
					}
				}
			});
		}
	} else if settings.accounts.len() > 1 {
		*info_msg.lock().unwrap() = Some("Select an account to continue".into());
	}

	enable_raw_mode()?;
	stdout().execute(EnterAlternateScreen)?;
	let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

	let picker = Picker::from_query_stdio().unwrap_or_else(|_| Picker::halfblocks());
	let mut input = String::new();
	let mut input_mode = false;

	let mut last_art_arc = None;
	let image_protocol = Arc::new(Mutex::new(None::<StatefulProtocol>));

	loop {
		if let Ok(event) = app_rx.try_recv() {
			match event {
				AppEvent::LoginSuccess(acc, provider) => {
					settings.active_account_id = Some(acc.id.clone());
					if !settings.accounts.iter().any(|a| a.id == acc.id) {
						settings.accounts.push(acc);
					}
					let _ = settings.save(None);
					core.clone().set_provider(Some(provider)).await;
					*info_msg.lock().unwrap() = None;
					mode = UiMode::Main;
				}
				AppEvent::LoginFailed(err) => {
					*info_msg.lock().unwrap() = None;
					*error_msg.lock().unwrap() = Some(err);
				}
			}
		}

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
			if mode == UiMode::Login {
				components::login::render_login(
					f,
					f.area(),
					&login_state,
					&settings.accounts,
					current_err.as_ref(),
					current_info.as_ref(),
				);
				return;
			}

			let main =
				Layout::vertical([Constraint::Min(0), Constraint::Length(6), Constraint::Length(3)]).split(f.area());
			let top = Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).split(main[0]);

			let art_area = Block::bordered().title(" Album Art ").inner(top[0]);
			f.render_widget(Block::bordered().title(" Album Art "), top[0]);

			if let Some(p) = prot_lock.as_mut() {
				f.render_stateful_widget(StatefulImage::default(), art_area, p);
			} else {
				let txt = if last_art_arc.is_some() { "Loading..." } else { "No Art" };
				f.render_widget(Paragraph::new(txt).alignment(Alignment::Center), art_area);
			}

			render_queue(f, top[1], &queue, pos);
			render_now_playing(f, main[1], current_track.as_ref(), &state.sync, state.scrobble_mark_pos);

			if let Some(e) = current_err {
				f.render_widget(
					Paragraph::new(e).red().block(Block::bordered().title(" Error ")),
					main[2],
				);
			} else if let Some(i) = current_info {
				f.render_widget(
					Paragraph::new(i).blue().block(Block::bordered().title(" Info ")),
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

				if mode == UiMode::Login {
					match components::login::handle_login_input(key, &mut login_state, &settings.accounts) {
						components::login::LoginAction::Connect(acc) => {
							*info_msg.lock().unwrap() = Some(format!("Connecting to {}...", acc.url));
							*error_msg.lock().unwrap() = None;
							let tx_c = app_tx.clone();
							tokio::spawn(async move {
								let entry = match keyring::Entry::new("musicbirb_subsonic", &acc.id) {
									Ok(e) => e,
									Err(e) => {
										let _ = tx_c.send(AppEvent::LoginFailed(format!("Keyring init error: {}", e)));
										return;
									}
								};
								let cred_str = match entry.get_password() {
									Ok(p) => p,
									Err(e) => {
										let _ = tx_c.send(AppEvent::LoginFailed(format!(
											"Keychain error (did you save it?): {}",
											e
										)));
										return;
									}
								};
								let credential = match serde_json::from_str::<AuthCredential>(&cred_str) {
									Ok(c) => c,
									Err(_) => AuthCredential::Password(cred_str.to_string()),
								};
								let authenticator = Authenticator::new();
								match authenticator
									.connect_with_credential(
										acc.provider.clone(),
										acc.url.clone(),
										acc.username.clone(),
										credential,
									)
									.await
								{
									Ok(provider) => {
										let _ = tx_c.send(AppEvent::LoginSuccess(acc, provider));
									}
									Err(e) => {
										let _ = tx_c.send(AppEvent::LoginFailed(format!("Provider error: {}", e)));
									}
								}
							});
						}
						components::login::LoginAction::ConnectNew(provider_id, url, user, pass) => {
							*info_msg.lock().unwrap() = Some("Validating...".into());
							*error_msg.lock().unwrap() = None;
							let tx_c = app_tx.clone();

							tokio::spawn(async move {
								let authenticator = Authenticator::new();

								match authenticator.init_auth(provider_id.clone(), url.clone()).await {
									Ok(AuthStep::UserPass) => {
										match authenticator
											.login_with_password(provider_id.clone(), url.clone(), user.clone(), pass)
											.await
										{
											Ok(result) => {
												let safe_id: String = format!("{}@{}", user, url)
													.chars()
													.map(|c| if c.is_alphanumeric() { c } else { '_' })
													.collect();

												let acc = musicbirb::settings::AccountConfig {
													id: safe_id,
													provider: provider_id.clone(),
													url: url.clone(),
													username: user.clone(),
												};

												if let Ok(cred_json) = serde_json::to_string(&result.credential) {
													if let Ok(entry) =
														keyring::Entry::new("musicbirb_subsonic", &acc.id)
													{
														let _ = entry.set_password(&cred_json);
													}
												}

												let _ = tx_c.send(AppEvent::LoginSuccess(acc, result.provider));
											}
											Err(e) => {
												let _ = tx_c.send(AppEvent::LoginFailed(e.to_string()));
											}
										}
									}
									Ok(AuthStep::BrowserAuth {
										auth_url,
										display_code: _,
										polling_id: _,
									}) => {
										// Needs UI logic (open browser window, poll backend), leaving placeholder
										let _ = tx_c.send(AppEvent::LoginFailed(format!(
											"Browser auth not fully implemented in TUI. Open: {}",
											auth_url
										)));
									}
									Err(e) => {
										let _ = tx_c.send(AppEvent::LoginFailed(format!("Init auth error: {}", e)));
									}
								}
							});
						}
						components::login::LoginAction::Delete(acc) => {
							if let Ok(entry) = keyring::Entry::new("musicbirb_subsonic", &acc.id) {
								let _ = entry.delete_credential();
							}
							settings.accounts.retain(|a| a.id != acc.id);
							if settings.active_account_id.as_ref() == Some(&acc.id) {
								settings.active_account_id = None;
							}
							let _ = settings.save(None);
							if login_state.selected_idx >= settings.accounts.len() {
								login_state.selected_idx = settings.accounts.len().saturating_sub(1);
							}
						}
						components::login::LoginAction::Quit => break,
						components::login::LoginAction::None => {}
					}
				} else if input_mode {
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
								if raw.trim() == "/logout" {
									settings.active_account_id = None;
									let _ = settings.save(None);
									let _ = core.clear_queue();
									tokio::spawn({
										let c = Arc::clone(&core);
										async move {
											c.set_provider(None).await;
										}
									});
									mode = UiMode::Login;
									login_state.focus = components::login::LoginFocus::List;
									continue;
								}

								let c = Arc::clone(&core);
								let i_m = Arc::clone(&info_msg);
								let e_m = Arc::clone(&error_msg);
								tokio::spawn(async move {
									*i_m.lock().unwrap() = Some("Fetching...".into());
									let res = if raw.starts_with("al:") {
										c.queue_album(AlbumId(raw.strip_prefix("al:").unwrap().to_string()), false)
											.await
											.map(|count| format!("Added {} tracks", count))
									} else if raw.starts_with("pl:") {
										c.queue_playlist(
											PlaylistId(raw.strip_prefix("pl:").unwrap().to_string()),
											false,
										)
										.await
										.map(|count| format!("Added {} tracks", count))
									} else {
										c.queue_track(TrackId(raw.clone()), false)
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
