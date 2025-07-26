use std::collections::HashMap;

use mlua::Lua;

use crate::ae2d::{bind, Camera::Drawable, Entity::Entity, Programmable::Programmable};

pub struct World
{
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
			script: Lua::new(),
			ents: vec![],
			prog: Programmable::new(),
			triggers: HashMap::new()
		}
	}

	pub fn load(&mut self, path: String)
	{
		self.script = Lua::new();
		self.ents.clear();
		
		let path = String::from("res/scripts/worlds/") + &path + ".lua";
		match self.script.load(std::fs::read_to_string(path).unwrap_or_default()).exec()
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
			// let n1 = self.ents[i].getName();
			// let id1 = self.ents[i].getID();
			// let h1 = self.ents[i].getHitbox();
			self.ents[i].update();
			// self.ents[i].applyVelocity(0);
			// do smth with this ^
			// for j in 0..self.ents.len()
			// {
			// 	if i >= j { continue; }
			// 	let h2 = self.ents[j].getHitbox();
			// 	if self.ents[i].checkCollision(h2)
			// 	{
			// 		let n2 = self.ents[j].getName();
			// 		let id2 = self.ents[j].getID();
			// 		self.ents[i].onCollided(
			// 			0, n2.clone(), id2.clone(),
			// 			h2.x, h2.y, h2.z, h2.w
			// 		);
			// 		self.ents[i].onCollided(
			// 			1, n2.clone(), id2.clone(),
			// 			h2.x, h2.y, h2.z, h2.w
			// 		);
			// 		self.ents[j].onCollided(
			// 			0, n1.clone(), id1.clone(),
			// 			h1.x, h1.y, h1.z, h1.w
			// 		);
			// 		self.ents[j].onCollided(
			// 			1, n1.clone(), id1.clone(),
			// 			h1.x, h1.y, h1.z, h1.w
			// 		);
			// 	}
			// }
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