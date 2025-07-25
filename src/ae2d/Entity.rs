use mlua::Lua;

use crate::ae2d::{bind, Camera::Drawable, Sprite::Sprite};

pub struct Entity
{
	script: Lua,
	id: String,
	name: String,
	group: String,
	sprite: Sprite
}

impl Entity
{
	pub fn new() -> Self
	{
		Self
		{
			script: Lua::new(),
			id: String::new(),
			name: String::new(),
			group: String::new(),
			sprite: Sprite::default()
		}
	}

	pub fn load(id: String, path: String) -> Self
	{
		let mut ent = Self::new();

		let src = json::parse(
			&std::fs::read_to_string(path).unwrap()
		).unwrap();

		for (var, value) in src.entries()
		{
			if var == "name"
			{
				ent.name = value.as_str().unwrap().to_string();
			}
			if var == "group"
			{
				ent.group = value.as_str().unwrap().to_string();
			}
			if var == "script"
			{
				let _ = ent.script.load(
					std::fs::read_to_string(
						value.as_str().unwrap()
					).unwrap()
				).exec();
			}
		}

		bind::sprite(&ent.script);
		bind::network(&ent.script);
		bind::world(&ent.script);
		bind::window(&ent.script);
		
		let _ = ent.script.globals().set(
			"ScriptID",
			format!("ent_{id}")
		);

		ent.id = id;

		ent
	}

	pub fn getSprite(&mut self) -> &mut Sprite
	{
		&mut self.sprite
	}

	pub fn init(&self, data: json::JsonValue)
	{
		let t = self.script.create_table().unwrap();
		
		for (var, value) in data.entries()
		{
			let _ = if value.is_number() { t.raw_set(var, value.as_f32().unwrap()) }
			else if value.is_boolean() { t.raw_set(var, value.as_bool().unwrap()) }
			else { t.raw_set(var, value.as_str().unwrap()) };
		}

		if let Ok(f) = self.script.globals().get::<mlua::Function>("Init")
		{
			let _ = f.call::<()>(t);
		}
	}

	pub fn update(&mut self)
	{
		bind::execFunc(&self.script, "Update");
	}

	pub fn getName(&self) -> String
	{
		self.name.clone()
	}

	pub fn getID(&self) -> String
	{
		self.id.clone()
	}
}

impl Drawable for Entity
{
	fn draw(&mut self)
	{
		bind::execFunc(&self.script, "Draw");
	}
}