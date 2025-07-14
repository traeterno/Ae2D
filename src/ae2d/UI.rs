use std::collections::HashMap;

use mlua::{Function, Lua, Value};

use super::{bind, Camera::Drawable, Sprite::Sprite, Text::Text, Window::Window};

pub struct Object
{
	script: Lua,
	spr: Sprite,
	text: Text
}

impl Object
{
	pub fn parse(name: &str, node: &json::JsonValue) -> Self
	{
		let mut obj = Self
		{
			script: Lua::new(),
			spr: Sprite::default(),
			text: Text::new()
		};

		let _ = obj.script.load_std_libs(mlua::StdLib::ALL_SAFE);
		let _ = obj.script.globals().set(
			"ScriptID",
			String::from("ui_") + name
		);

		bind::sprite(&obj.script);
		bind::text(&obj.script);
		Window::initLua(&obj.script);

		for (var, value) in node.entries()
		{
			if var == "script"
			{
				let _ = obj.script.load(
					std::fs::read_to_string(
						value.as_str().unwrap()
					).unwrap_or(String::new())
				).exec();
			}
			if var == "image"
			{
				obj.spr = Sprite::image(
					value.as_str().unwrap().to_string()
				);
			}
			if var == "anim"
			{
				obj.spr = Sprite::animated(
					value.as_str().unwrap().to_string()
				);
			}
			if var == "text"
			{
				for (x, y) in value.entries()
				{
					if x == "font"
					{
						obj.text.setFont(
							y.as_str().unwrap().to_string()
						);
					}
					if x == "size"
					{
						obj.text.setSize(
							y.as_f32().unwrap()
						);
					}
					if x == "text"
					{
						obj.text.setString(
							y.as_str().unwrap().to_string()
						);
					}
				}
			}
			if var == "vars"
			{
				let t = obj.script.create_table().unwrap();
				for (x, y) in value.entries()
				{
					match y.as_f32()
					{
						Some(v) =>
						{
							let _ = t.raw_set(x, v);
						}
						None =>
						{
							let _ = t.raw_set(
								x, y.as_str().unwrap()
							);
						}
					};
				}
				let _ = obj.script.globals().set("vars", t);
			}
		}

		obj
	}

	pub fn getSprite(&mut self) -> &mut Sprite
	{
		&mut self.spr
	}

	pub fn getText(&mut self) -> &mut Text
	{
		&mut self.text
	}
}

pub struct UI
{
	baseSize: glam::Vec2,
	objects: HashMap<String, Object>,
	reload: String
}

impl UI
{
	pub fn new() -> Self
	{
		Self
		{
			baseSize: glam::Vec2::ZERO,
			objects: HashMap::new(),
			reload: String::new()
		}
	}

	pub fn setSize(&mut self, size: glam::Vec2)
	{
		self.baseSize = size;
	}

	pub fn load(&mut self, path: String)
	{
		let src = json::parse(
			&std::fs::read_to_string(path)
			.unwrap_or(String::new())
		);
		if src.is_err()
		{
			println!("Failed to load UI: {}", src.unwrap_err());
			return;
		}
		let src = src.unwrap();

		self.objects.clear();

		for (name, value) in src.entries()
		{
			self.objects.insert(
				name.to_string(),
				Object::parse(name, value)
			);
		}

		for (name, obj) in &self.objects
		{
			if let Ok(f) = obj.script.globals()
				.get::<mlua::Function>("Init")
			{
				match f.call::<mlua::Value>(())
				{
					Ok(_) => {},
					Err(x) =>
					{
						println!("Object: {name}\n{x}\n");
					}
				}
			}
		}
	}

	pub fn getObject(&mut self, name: String) -> &mut Object
	{
		self.objects.get_mut(&name).unwrap()
	}

	pub fn update(&mut self)
	{
		if !self.reload.is_empty()
		{
			self.load(self.reload.clone());
			self.reload.clear();
		}
		for (name, obj) in &self.objects
		{
			if let Ok(f) = obj.script.globals()
				.get::<mlua::Function>("Update")
			{
				match f.call::<mlua::Value>(())
				{
					Ok(_) => {},
					Err(x) =>
					{
						println!("Object: {name}\n{x}\n");
					}
				}
			}
		}
	}

	pub fn requestLoad(&mut self, path: String)
	{
		self.reload = path;
	}

	pub fn resize(&mut self, w: i32, h: i32)
	{
		for (_, obj) in &self.objects
		{
			if let Ok(f) = obj.script.globals().get::<Function>("OnResized")
			{
				let _ = f.call::<Value>((w, h));
			}
		}
	}
	
	pub fn getSize(&self) -> glam::Vec2 { self.baseSize }
}

impl Drawable for UI
{
	fn draw(&mut self)
	{
		for (name, obj) in &self.objects
		{
			if let Ok(f) = obj.script.globals()
				.get::<mlua::Function>("Draw")
			{
				match f.call::<mlua::Value>(())
				{
					Ok(_) => {},
					Err(x) =>
					{
						println!("Object: {name}\n{x}\n");
					}
				}
			}
		}
	}
}