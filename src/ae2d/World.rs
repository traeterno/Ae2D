use std::collections::HashMap;

use mlua::Lua;

use crate::ae2d::{bind, Camera::Drawable, Entity::Entity, Programmable::Programmable};

pub struct World
{
	name: String,
	script: Lua,
	ents: Vec<Entity>,
	prog: Programmable,
	triggers: HashMap<String, (String, glam::Vec4)>
}

impl World
{
	pub fn new() -> Self
	{
		Self
		{
			name: String::new(),
			script: Lua::new(),
			ents: vec![],
			prog: Programmable::new(),
			triggers: HashMap::new()
		}
	}

	pub fn load(&mut self, id: String)
	{
		let path = String::from("res/scripts/worlds/") + &id + ".lua";
		self.parse(
			id,
			std::fs::read_to_string(path).unwrap_or_default()
		);
	}

	pub fn parse(&mut self, id: String, src: String)
	{
		self.name = id;
		self.script = Lua::new();
		self.ents.clear();

		match self.script.load(src).exec()
		{
			Ok(_) => {}
			Err(x) => { println!("Не удалось загрузить мир: {x}"); return; }
		}

		bind::window(&self.script);
		bind::network(&self.script);
		bind::world(&self.script);
		bind::execFunc(&self.script, "Init"); 
	}

	pub fn update(&mut self)
	{
		bind::execFunc(&self.script, "Update");
		for i in 0..self.ents.len()
		{
			self.ents[i].update();
		}
	}

	pub fn getEntity(&mut self, id: String) -> &mut Entity
	{
		for e in &mut self.ents
		{
			if e.getID() == id { return e; }
		}
		panic!("Entity '{id}' not found");
	}

	pub fn spawn(&mut self, id: String, path: String, vars: json::JsonValue)
	{
		self.ents.push(Entity::load(id, path));
		self.ents.last().unwrap().init(vars);
	}

	pub fn kill(&mut self, id: String)
	{
		for i in 0..self.ents.len()
		{
			if self.ents[i].getID() == id
			{
				self.ents.remove(i);
				return;
			}
		}
	}

	pub fn createTrigger(&mut self, id: String, name: String, hitbox: glam::Vec4)
	{
		self.triggers.insert(id, (name, hitbox));
	}

	pub fn modifyTrigger(&mut self, id: String, hitbox: glam::Vec4)
	{
		if let Some(t) = self.triggers.get_mut(&id)
		{
			t.1 = hitbox;
		}
	}

	pub fn getTriggers(&self) -> &HashMap<String, (String, glam::Vec4)>
	{
		&self.triggers
	}

	pub fn getProgrammable(&mut self) -> &mut Programmable
	{
		&mut self.prog
	}

	pub fn getName(&self) -> String { self.name.clone() }
}

impl Drawable for World
{
	fn draw(&mut self)
	{
		for ent in &mut self.ents
		{
			ent.draw();
		}
	}
}