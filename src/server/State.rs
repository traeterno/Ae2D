use std::{collections::HashMap, net::IpAddr};

use crate::server::Config::Config;

pub struct State
{
	pub playersList: HashMap<IpAddr, (String, String)>,
	pub checkpoint: String,
	pub date: String,
	pub chatHistory: Vec<(String, String)>
}

impl State
{
	fn new() -> Self
	{
		Self
		{
			playersList: HashMap::new(),
			checkpoint: String::new(),
			date: String::new(),
			chatHistory: vec![],
		}
	}
	fn load(file: String, cfg: &Config) -> Self
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
					}

					state.playersList.insert(
						ip.parse().unwrap(),
						(name, class)
					);
				}
			}
			if section.0 == "checkpoint"
			{
				state.checkpoint = section.1.as_str().unwrap().to_string();
			}
			if section.0 == "date"
			{
				state.date = section.1.as_str().unwrap_or("").to_string();
			}
		}
		
		if state.checkpoint.is_empty()
		{
			state.checkpoint = cfg.firstCheckpoint.clone();
		}
		
		state
	}

	pub fn init(cfg: &Config) -> Self
	{
		match std::fs::read_to_string("res/system/save.json")
		{
			Ok(file) => Self::load(file, cfg),
			Err(_) => Self::new()
		}
	}

	pub fn save(&mut self, checkpoint: String)
	{
		self.date = State::getDateTime();
		self.checkpoint = checkpoint;

		let mut players = json::JsonValue::new_object();
		for (ip, data) in &self.playersList
		{
			let mut info = json::object! {};
			let name = data.0.clone();
			let _ = info.insert("name", name.clone());
			let _ = info.insert("class", data.1.clone());
			let _ = players.insert(&ip.to_string(), info);
		}

		let state = json::object!
		{
			players: players,
			date: self.date.clone(),
			checkpoint: self.checkpoint.clone()
		};

		let _ = std::fs::write(
			"res/system/save.json",
			json::stringify_pretty(state, 4)
		);
	}

	pub fn getPlayerInfo(&mut self, ip: IpAddr) -> (String, String)
	{
		match self.playersList.get(&ip)
		{
			Some(data) => data.clone(),
			None => (String::from("noname"), String::from("unknown"))
		}
	}
	
	pub fn setPlayerInfo(&mut self, ip: IpAddr, name: String, class: String)
	{
		self.playersList.insert(ip, (name, class));
	}

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
}