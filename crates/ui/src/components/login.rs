use crossterm::event::{KeyCode, KeyEvent};
use musicbirb::settings::AccountConfig;
use ratatui::{prelude::*, widgets::*};

pub enum LoginAction {
	None,
	Connect(AccountConfig),
	ConnectNew(String, String, String),
	Delete(AccountConfig),
	Quit,
}

#[derive(PartialEq)]
pub enum LoginFocus {
	List,
	Url,
	User,
	Pass,
}

pub struct LoginState {
	pub focus: LoginFocus,
	pub selected_idx: usize,
	pub url: String,
	pub user: String,
	pub pass: String,
}

impl Default for LoginState {
	fn default() -> Self {
		Self {
			focus: LoginFocus::List,
			selected_idx: 0,
			url: String::new(),
			user: String::new(),
			pass: String::new(),
		}
	}
}

pub fn render_login(
	f: &mut Frame,
	area: Rect,
	state: &LoginState,
	accounts: &[AccountConfig],
	err: Option<&String>,
	info: Option<&String>,
) {
	let chunks = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
		.margin(4)
		.split(area);

	// Left side: Accounts List
	let mut items = vec![];
	for (i, acc) in accounts.iter().enumerate() {
		let style = if state.focus == LoginFocus::List && state.selected_idx == i {
			Style::default().fg(Color::Yellow).bold()
		} else {
			Style::default()
		};
		items.push(ListItem::new(format!("{} ({})", acc.username, acc.url)).style(style));
	}
	let list_style = if state.focus == LoginFocus::List && state.selected_idx == accounts.len() {
		Style::default().fg(Color::Yellow).bold()
	} else {
		Style::default()
	};
	items.push(ListItem::new("+ Add New Account").style(list_style));

	let list = List::new(items).block(Block::bordered().title(" Saved Accounts (Enter: Connect, D: Delete) "));
	f.render_widget(list, chunks[0]);

	// Right side: Add New
	let right_chunks = Layout::vertical([
		Constraint::Length(3),
		Constraint::Length(3),
		Constraint::Length(3),
		Constraint::Length(3),
		Constraint::Min(0),
	])
	.split(chunks[1]);

	let url_style = if state.focus == LoginFocus::Url {
		Style::default().fg(Color::Yellow)
	} else {
		Style::default()
	};
	f.render_widget(
		Paragraph::new(state.url.clone())
			.block(Block::bordered().title(" Server URL "))
			.style(url_style),
		right_chunks[0],
	);

	let user_style = if state.focus == LoginFocus::User {
		Style::default().fg(Color::Yellow)
	} else {
		Style::default()
	};
	f.render_widget(
		Paragraph::new(state.user.clone())
			.block(Block::bordered().title(" Username "))
			.style(user_style),
		right_chunks[1],
	);

	let pass_style = if state.focus == LoginFocus::Pass {
		Style::default().fg(Color::Yellow)
	} else {
		Style::default()
	};
	let pass_display = "*".repeat(state.pass.len());
	f.render_widget(
		Paragraph::new(pass_display)
			.block(Block::bordered().title(" Password "))
			.style(pass_style),
		right_chunks[2],
	);

	if let Some(e) = err {
		f.render_widget(Paragraph::new(e.clone()).red(), right_chunks[3]);
	} else if let Some(i) = info {
		f.render_widget(Paragraph::new(i.clone()).blue(), right_chunks[3]);
	} else {
		f.render_widget(
			Paragraph::new("Press Tab to navigate, Enter to submit, Esc to List").gray(),
			right_chunks[3],
		);
	}
}

pub fn handle_login_input(key: KeyEvent, state: &mut LoginState, accounts: &[AccountConfig]) -> LoginAction {
	match state.focus {
		LoginFocus::List => match key.code {
			KeyCode::Up => {
				if state.selected_idx > 0 {
					state.selected_idx -= 1;
				}
			}
			KeyCode::Down => {
				if state.selected_idx < accounts.len() {
					state.selected_idx += 1;
				}
			}
			KeyCode::Char('d') | KeyCode::Char('D') => {
				if state.selected_idx < accounts.len() {
					return LoginAction::Delete(accounts[state.selected_idx].clone());
				}
			}
			KeyCode::Enter => {
				if state.selected_idx == accounts.len() {
					state.focus = LoginFocus::Url;
				} else {
					return LoginAction::Connect(accounts[state.selected_idx].clone());
				}
			}
			KeyCode::Esc => return LoginAction::Quit,
			_ => {}
		},
		LoginFocus::Url | LoginFocus::User | LoginFocus::Pass => match key.code {
			KeyCode::Esc => {
				state.focus = LoginFocus::List;
			}
			KeyCode::Tab | KeyCode::Down => {
				state.focus = match state.focus {
					LoginFocus::Url => LoginFocus::User,
					LoginFocus::User => LoginFocus::Pass,
					_ => LoginFocus::Url,
				};
			}
			KeyCode::BackTab | KeyCode::Up => {
				state.focus = match state.focus {
					LoginFocus::Url => LoginFocus::Pass,
					LoginFocus::User => LoginFocus::Url,
					_ => LoginFocus::User,
				};
			}
			KeyCode::Backspace => match state.focus {
				LoginFocus::Url => {
					state.url.pop();
				}
				LoginFocus::User => {
					state.user.pop();
				}
				LoginFocus::Pass => {
					state.pass.pop();
				}
				_ => {}
			},
			KeyCode::Char(c) => match state.focus {
				LoginFocus::Url => {
					state.url.push(c);
				}
				LoginFocus::User => {
					state.user.push(c);
				}
				LoginFocus::Pass => {
					state.pass.push(c);
				}
				_ => {}
			},
			KeyCode::Enter => {
				if !state.url.is_empty() && !state.user.is_empty() && !state.pass.is_empty() {
					let action = LoginAction::ConnectNew(state.url.clone(), state.user.clone(), state.pass.clone());
					state.pass.clear();
					return action;
				}
			}
			_ => {}
		},
	}
	LoginAction::None
}
