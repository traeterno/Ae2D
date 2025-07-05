use std::{collections::HashMap, time::Duration};

#[derive(Clone, PartialEq)]
pub enum Permission
{
	Developer,
	Admin,
	Player
}

impl Permission
{
	pub fn fromString(x: &str) -> Self
	{
		if x == "dev" { return Permission::Developer; }
		if x == "admin" { return Permission::Admin; }
		Permission::Player
	}

	pub fn toString(&self) -> String
	{
		match self
		{
			Permission::Developer => String::from("dev"),
			Permission::Admin => String::from("admin"),
			Permission::Player => String::from("player")
		}
	}

	pub fn check(&self, lvl: Permission) -> bool
	{
		match lvl
		{
			Permission::Player => true,
			Permission::Admin => { *self == Permission::Admin || *self == Permission::Developer },
			Permission::Developer => *self == Permission::Developer
		}
	}
}

pub struct Config
{
	pub maxPlayersCount: u8,
	pub port: u16,
	pub tickRate: u8,
	pub sendTime: Duration,
	pub recvTime: Duration,
	pub permissions: HashMap<String, Permission>,
}

impl Default for Config
{
	fn default() -> Self
	{
		Self
		{
			maxPlayersCount: 5,
			port: 0,
			tickRate: 1,
			sendTime: Duration::from_secs(1),
			recvTime: Duration::from_secs_f32(0.5),
			permissions: HashMap::new()
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
					if name == "maxPlayersCount"
					{
						state.maxPlayersCount = value.as_u8().unwrap_or(1);
					}
					if name == "port"
					{
						state.port = value.as_u16().unwrap_or(2018);
					}
					if name == "tickRate"
					{
						state.tickRate = value.as_u8().unwrap_or(30);
						state.sendTime = Duration::from_secs_f32(1.0 / state.tickRate as f32);
						state.recvTime = Duration::from_secs_f32(0.5 / state.tickRate as f32);
					}
				}
			}
			if section.0 == "permissions"
			{
				for (name, group) in section.1.entries()
				{
					state.permissions.insert(
						name.to_string(),
						Permission::fromString(group.as_str().unwrap_or(""))
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
				println!("Failed to load config: {:?}\nCreating new config.", error);
				Self::default()
			}
		}
	}

	pub fn save(&self)
	{
		let mut settings = json::JsonValue::new_object();
		let _ = settings.insert("maxPlayersCount", self.maxPlayersCount);
		let _ = settings.insert("port", self.port);
		let _ = settings.insert("tickRate", self.tickRate);

		let mut permissions = json::JsonValue::new_object();
		for (name, group) in &self.permissions
		{
			let _ = permissions.insert(&name, group.toString());
		}
		
		let mut state = json::JsonValue::new_object();
		let _ = state.insert("settings", settings);
		let _ = state.insert("permissions", permissions);
		
		let _ = std::fs::write("res/system/config.json", json::stringify_pretty(state, 4));
	}

	pub fn getPermission(&mut self, name: &String) -> Permission
	{
		if name == "WebClient" { return Permission::Developer; }
		self.permissions.get(name).unwrap_or(&Permission::Player).clone()
	}

	pub fn setPermission(&mut self, name: String, group: Permission)
	{
		self.permissions.insert(name, group);
	}
}