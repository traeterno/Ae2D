use mlua::Lua;

use crate::ae2d::{bind, Camera::Drawable, Sprite::Sprite};

pub struct Entity
{
	script: Lua,
	id: String,
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
			sprite: Sprite::default()
		}
	}

	pub fn load(id: String, path: String) -> Self
	{
		let mut ent = Self::new();
		
		bind::sprite(&ent.script);
		bind::network(&ent.script);
		bind::world(&ent.script);
		bind::window(&ent.script);
		bind::shapes(&ent.script);
		bind::shaders(&ent.script);

		let _ = ent.script.load(
			std::fs::read_to_string(
				path
			).unwrap()
		).exec();
		
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

	pub fn update(&mut self) -> (u8, bool)
	{
		if let Ok(f) = self.script.globals().get::<mlua::Function>("Update")
		{
			if let Ok(x) = f.call::<(u8, bool)>(())
			{
				return x;
			}
		}
		(0, true)
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