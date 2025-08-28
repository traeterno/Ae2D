use std::{collections::HashMap, net::IpAddr};

use crate::server::Server::Server;

#[derive(Clone, Debug)]
pub struct Account
{
	pub name: String,
	pub class: String,
	pub color: (u8, u8, u8),
	pub hp: u16
}

impl Default for Account
{
	fn default() -> Self
	{
		Self
		{
			name: String::from("noname"),
			class: String::from("unknown"),
			color: (255u8, 255u8, 255u8),
			hp: 0
		}
	}
}

pub struct Settings
{
	pub extendPlayers: bool,
	pub tickRate: u8,
	pub firstCP: String,
	pub maxItemCellSize: u8,
	pub sendTime: std::time::Duration
}

pub struct Save
{
	pub checkpoint: String,
	pub date: String,
}

pub struct State
{
	pub accounts: HashMap<IpAddr, Account>,
	pub chatHistory: Vec<(String, String)>,
	pub settings: Settings,
	pub save: Save
}

impl State
{
	fn new() -> Self
	{
		Self
		{
			accounts: HashMap::new(),
			chatHistory: vec![],
			settings: Settings
			{
				extendPlayers: false, tickRate: 1, firstCP: String::from("main"),
				maxItemCellSize: 64,
				sendTime: std::time::Duration::from_secs(1)
			},
			save: Save { checkpoint: String::from("main"), date: String::new() }
		}
	}
	fn load(file: String) -> Self
	{
		let doc = json::parse(&file);
		if doc.is_err() { println!("Failed to load save."); return Self::new(); }
		let doc = doc.unwrap();
		let mut state = Self::new();

		for section in doc.entries()
		{
			if section.0 == "players"
			{
				for (ip, player) in section.1.entries()
				{
					let mut name = String::new();
					let mut class = String::new();
					let mut color = (255, 255, 255);
					for arg in player.entries()
					{
						if arg.0 == "name"
						{
							name = arg.1.as_str().unwrap_or("").to_string();
						}
						if arg.0 == "class"
						{
							class = arg.1.as_str().unwrap_or("").to_string();
						}
						if arg.0 == "color"
						{
							for c in arg.1.entries()
							{
								if c.0 == "r" { color.0 = c.1.as_u8().unwrap(); }
								if c.0 == "g" { color.1 = c.1.as_u8().unwrap(); }
								if c.0 == "b" { color.2 = c.1.as_u8().unwrap(); }
							}
						}
					}

					state.accounts.insert(
						ip.parse().unwrap(),
						Account { name, class, color, hp: 0 }
					);
				}
			}
			if section.0 == "save"
			{
				for (var, value) in section.1.entries()
				{
					if var == "checkpoint"
					{
						state.save.checkpoint = value.as_str().unwrap().to_string();
					}
					if var == "date"
					{
						state.save.date = value.as_str().unwrap().to_string();
					}
				}
			}
			if section.0 == "settings"
			{
				for (var, value) in section.1.entries()
				{
					if var == "extendPlayers"
					{
						state.settings.extendPlayers = value.as_bool().unwrap();
					}
					if var == "tickRate"
					{
						state.settings.tickRate = value.as_u8().unwrap();
						state.settings.sendTime = std::time::Duration::from_secs_f32(
							1.0 / state.settings.tickRate as f32
						);
					}
					if var == "firstCP"
					{
						state.settings.firstCP = value.as_str().unwrap().to_string();
					}
					if var == "maxItemCellSize"
					{
						state.settings.maxItemCellSize = value.as_u8().unwrap();
					}
				}
			}
		}
		
		if state.save.checkpoint.is_empty()
		{
			state.save.checkpoint = state.settings.firstCP.clone();
		}
		
		state
	}

	pub fn init() -> Self
	{
		match std::fs::read_to_string("res/system/save.json")
		{
			Ok(file) => Self::load(file),
			Err(_) => Self::new()
		}
	}

	pub fn save(&mut self, checkpoint: String)
	{
		self.save.date = State::getDateTime();
		self.save.checkpoint = checkpoint;

		let mut players = json::JsonValue::new_object();
		for (ip, data) in &self.accounts
		{
			let _ = players.insert(&ip.to_string(), json::object!
			{
				name: data.name.clone(),
				class: data.class.clone(),
				color: {
					r: data.color.0,
					g: data.color.1,
					b: data.color.2
				}
			});
		}
		for (_, c) in Server::getPlayers()
		{
			if c.tcp.is_none() { continue; }
			let ip = c.tcp.as_ref().unwrap().peer_addr().unwrap().ip();
			let _ = players.insert(&ip.to_string(), json::object!{
				name: c.info.name.clone(),
				class: c.info.class.clone(),
				color: {
					r: c.info.color.0,
					g: c.info.color.1,
					b: c.info.color.2
				}
			});
		}

		let state = json::object!
		{
			players: players,
			save: {
				date: self.save.date.clone(),
				checkpoint: self.save.checkpoint.clone()
			},
			settings: {
				extendPlayers: self.settings.extendPlayers,
				tickRate: self.settings.tickRate,
				firstCP: self.settings.firstCP.clone(),
				maxItemCellSize: self.settings.maxItemCellSize
			}
		};

		let _ = std::fs::write(
			"res/system/save.json",
			json::stringify_pretty(state, 4)
		);
	}

	pub fn getPlayerInfo(&mut self, ip: IpAddr) -> Account
	{
		if let None = self.accounts.get_mut(&ip)
		{
			self.accounts.insert(ip.clone(), Account::default());
		}
		self.accounts.get(&ip).unwrap().clone()
	}
	
	// pub fn updateAccounts(&mut self, ip: IpAddr, pi: Account)
	// {
	// 	self.accounts.insert(ip, pi);
	// }

	pub fn getDateTime() -> String
	{
		match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
		{
			Ok(t) =>
			{
				let seconds = t.as_secs();
				let minutes = seconds / 60; let seconds = seconds % 60;
				let hours = minutes / 60; let minutes = minutes % 60;
				let mut days = hours / 24; let hours = hours % 24;

				let mut years = 1970 + days / 1461 * 4; days = days % 1461;
				while days > 365 { years = years + 1; days = days - 365; }

				let mut month = 1;
				'getMonth: loop
				{
					if (month == 0 || month == 2 || month == 4 ||
						month == 6 || month == 7 || month == 9 ||
						month == 11 || month == 12) && days > 31 { month += 1; days -= 31; }
					else if month == 1
					{
						if years % 4 == 0 && days > 29 { month += 1; days -= 29; }
						else if years % 4 != 0 && days > 28 { month += 1; days -= 28; }
					}
					else if (month == 3 || month == 5 || month == 8 || month == 10) && days > 30
					{
						month += 1; days -= 30;
					}
					else { break 'getMonth; }
				}

				let m = String::from(match month
				{
					1 => "Января",
					2 => "Февраля",
					3 => "Марта",
					4 => "Апреля",
					5 => "Мая",
					6 => "Июня",
					7 => "Июля",
					8 => "Августа",
					9 => "Сентября",
					10 => "Октября",
					11 => "Ноября",
					12 => "Декабря",
					_ => "???"
				});
				
				return format!("{days} {m} {years} - {hours}:{minutes}:{seconds}");
			},
			Err(_) => { return String::new(); }
		}
	}

	pub fn getPlayersCount(&self) -> u8
	{
		5 * if self.settings.extendPlayers { 2 } else { 1 }
	}
}