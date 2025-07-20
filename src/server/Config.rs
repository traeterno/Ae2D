use std::{collections::HashMap, time::Duration};

pub enum Permission
{
	Developer,
	Player
}

impl Permission
{
	pub fn toString(&self) -> String
	{
		match self
		{
			Permission::Developer => String::from("dev"),
			Permission::Player => String::from("player")
		}
	}

	pub fn check(&self, lvl: Permission) -> bool
	{
		match lvl
		{
			Permission::Player => true,
			Permission::Developer => match *self
			{
				Permission::Developer => true,
				Permission::Player => false
			}
		}
	}
}

pub struct Config
{
	pub extendedPlayers: bool,
	pub tickRate: u8,
	pub sendTime: Duration,
	pub recvTime: Duration,
	pub permissions: HashMap<String, Permission>,
	pub firstCheckpoint: String
}

impl Default for Config
{
	fn default() -> Self
	{
		Self
		{
			extendedPlayers: false,
			tickRate: 1,
			sendTime: Duration::from_secs(1),
			recvTime: Duration::from_secs_f32(0.5),
			permissions: HashMap::new(),
			firstCheckpoint: String::from("main")
		}
	}
}

impl Config
{
	fn load(file: String) -> Self
	{
		let doc = json::parse(&file);
		if doc.is_err()
		{
			println!("Failed to load config: {}", doc.unwrap_err());
			return Self::default();
		}
		let doc = doc.unwrap();
		let mut state = Self::default();

		for section in doc.entries()
		{
			if section.0 == "settings"
			{
				for (name, value) in section.1.entries()
				{
					if name == "extendedPlayers"
					{
						state.extendedPlayers = value.as_bool().unwrap();
					}
					if name == "tickRate"
					{
						state.tickRate = value.as_u8().unwrap();
						state.sendTime = Duration::from_secs_f32(1.0 / state.tickRate as f32);
						state.recvTime = Duration::from_secs_f32(0.5 / state.tickRate as f32);
					}
					if name == "firstCP"
					{
						state.firstCheckpoint = value.as_str().unwrap().to_string();
					}
				}
			}
			if section.0 == "permissions"
			{
				for (name, group) in section.1.entries()
				{
					state.permissions.insert(
						name.to_string(),
						match group.as_str().unwrap_or("player")
						{
							"dev" => Permission::Developer,
							"player" => Permission::Player,
							x => panic!("Wrong permission level: {x}")
						}
					);
				}
			}
		}
		
		state
	}

	pub fn init() -> Self
	{
		match std::fs::read_to_string("res/system/config.json")
		{
			Ok(file) => { Self::load(file) },
			Err(error) =>
			{
				println!("Failed to load config: {}. Creating new config.", error);
				Self::default()
			}
		}
	}

	pub fn save(&self)
	{
		let mut permissions = json::JsonValue::new_object();
		for (name, group) in &self.permissions
		{
			let _ = permissions.insert(&name, group.toString());
		}
		
		let mut state = json::JsonValue::new_object();
		let _ = state.insert("permissions", permissions);

		let _ = state.insert("settings", json::object!
		{
			extendedPlayers: self.extendedPlayers,
			tickRate: self.tickRate,
			firstCP: self.firstCheckpoint.clone(),
		});
		
		match std::fs::write("res/system/config.json", json::stringify_pretty(state, 4))
		{
			Ok(_) => {},
			Err(x) => match x.kind()
			{
				std::io::ErrorKind::NotFound =>
				{
					let _ = std::fs::DirBuilder::new().recursive(true).create("res/system");
					self.save();
				}
				_ => {}
			}
		}
	}

	pub fn getPermission(&mut self, name: &String) -> &Permission
	{
		if name == "WebClient" { return &Permission::Developer; }
		self.permissions.get(name).unwrap_or(&Permission::Player)
	}

	pub fn setPermission(&mut self, name: String, group: Permission)
	{
		self.permissions.insert(name, group);
	}

	pub fn getPlayersCount(&self) -> u8 { 5 * if self.extendedPlayers { 2 } else { 1 } }
}