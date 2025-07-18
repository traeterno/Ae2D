use std::ptr::null;

use mlua::{Error, Function, Lua, StdLib, Value::{self}};

use crate::ae2d::{Camera::Drawable, Network::Network, Programmable::{Programmable, Variable}, Window::Window};

use super::Entity::Entity;

pub struct World
{
	ents: Vec<Entity>,
	script: Lua,
	currentEnt: *mut Entity,
	updateFN: Option<Function>,
	pub prog: Programmable
}

impl World
{
	pub fn new() -> Self
	{
		let world = Self
		{
			ents: vec![],
			script: Lua::new(),
			currentEnt: null::<Entity>() as *mut _,
			updateFN: None,
			prog: Programmable::new()
		};
		
		world.script.load_std_libs(StdLib::ALL_SAFE);
		Network::initLua(&world.script);
		World::initLua(&world.script);
		
		world
	}

	pub fn initLua(script: &Lua)
	{
		let table = script.create_table().unwrap();
		table.set("setNum", script.create_function(World::setNumFN).unwrap());
		table.set("setStr", script.create_function(World::setStrFN).unwrap());
		table.set("getNum", script.create_function(World::getNumFN).unwrap());
		table.set("getStr", script.create_function(World::getStrFN).unwrap());
		table.set("hitbox", script.create_function(World::hitboxFN).unwrap());
		table.set("execute", script.create_function(World::executeFN).unwrap());
		script.globals().set("world", table);
	}

	pub fn load(&mut self, path: String)
	{
		let src = json::parse(
			&std::fs::read_to_string(path)
			.unwrap_or(String::new())
		);
		if src.is_err() { return; }
		let src = src.unwrap();

		self.ents.clear();

		for (var, value) in src.entries()
		{
			if var == "script"
			{
				self.script.load(
					std::fs::read_to_string(value.as_str().unwrap())
					.unwrap()
				).exec();
			}
			if var == "entities"
			{
				for (id, file) in value.entries()
				{
					self.ents.push(
						Entity::load(id, file.as_str().unwrap())
					);
				}
			}
		}
				
		if let Ok(func) = self.script.globals().get::<Function>("Init")
		{
			func.call::<Value>(());
		}

		// let doc = Assets::readXML(path);
		// if doc.is_none() { return; }
		// let doc = doc.unwrap();

		// self.script.load(Assets::readFile(
		// 	doc.att_opt("script")
		// 	.unwrap_or("")
		// 	.to_string()
		// ).unwrap()).exec();

		// for el in doc.elements()
		// {
		// 	if el.name().local_part() == "entity"
		// 	{
		// 		let mut ent = Entity::load(
		// 			el.att_opt("path").unwrap_or("").to_string()
		// 		);

		// 		let mut prog = Programmable::new();
		// 		for var in el.elements()
		// 		{
		// 			if var.name().local_part() == "var"
		// 			{
		// 				prog.insert(
		// 					var.att_opt("name").unwrap_or("").to_string(),
		// 					Variable
		// 					{
		// 						num: var.att_opt("num").unwrap_or("0").parse().unwrap(),
		// 						string: var.att_opt("str").unwrap_or("").to_string()
		// 					}
		// 				);
		// 			}
		// 		}

		// 		self.currentEnt = &mut ent;

		// 		ent.init(
		// 			el.att_opt("id").unwrap_or("").to_string(),
		// 			prog
		// 		);

		// 		self.ents.push(ent);
		// 	}
		// }

		if let Ok(func) = self.script.globals().get::<Function>("Update")
		{
			self.updateFN = Some(func);
		}
	}

	pub fn update(&mut self)
	{
		if let Some(func) = &self.updateFN
		{
			func.call::<Value>(());
		}

		for ent in &mut self.ents
		{
			self.currentEnt = ent;
			ent.prePhysics();
		}

		for dir in 0..2
		{
			for i in 0..self.ents.len()
			{
				let n1 = self.ents[i].getName().clone();
				let id1 = self.ents[i].getID().clone();
				for j in i..self.ents.len()
				{
					if j == i { continue; }
					let h = self.ents[j].getRB().getHitbox();
					if self.ents[i].getRB().checkCollision(h)
					{
						let n2 = self.ents[j].getName().clone();
						let id2 = self.ents[j].getName().clone();
						let h1 = self.ents[i].getRB().getHitbox();
						self.ents[i].onCollided(dir, n2.clone(), id2.clone(), h);
						self.ents[j].onCollided(dir, n1.clone(), id1.clone(), h1);
					}
				}
				if dir == 0 { self.ents[i].midPhysics(); }
			}
		}

		for ent in &mut self.ents
		{
			self.currentEnt = ent;
			ent.postPhysics();
		}
	}

	pub fn render(&mut self)
	{
		for ent in &mut self.ents
		{
			ent.draw();
		}
	}

	pub fn getCurrentEntity(&mut self) -> &mut Entity
	{
		unsafe { self.currentEnt.as_mut().unwrap() }
	}

	pub fn setCurrentEntity(&mut self, ent: &mut Entity)
	{
		self.currentEnt = ent;
	}

	pub fn getEntity(&mut self, name: String) -> Option<&mut Entity>
	{
		for e in &mut self.ents
		{
			if *e.getID() == name { return Some(e); }
		}
		None
	}

	pub fn executeFN(_: &Lua, data: (String, String)) -> Result<(), Error>
	{
		if let Some(e) = Window::getWorld().getEntity(data.0)
		{
			Window::getWorld().setCurrentEntity(e);
			e.execute(data.1);
		}
		Ok(())
	}

	pub fn setNumFN(_: &Lua, args: (String, f32)) -> Result<(), Error>
	{
		let prog = &mut Window::getWorld().prog;
		let var = prog.get_mut(&args.0);
		if let Some(x) = var { x.num = args.1; }
		else { prog.insert(args.0, Variable { num: args.1, string: String::new() }); }
		Ok(())
	}

	pub fn getNumFN(_: &Lua, args: String) -> Result<f64, Error>
	{
		let prog = &mut Window::getWorld().prog;
		let var = prog.get(&args);
		if var.is_none() { return Ok(0.0); }
		Ok(var.unwrap().num as f64)
	}

	pub fn setStrFN(_: &Lua, args: (String, String)) -> Result<(), Error>
	{
		let prog = &mut Window::getWorld().prog;
		let var = prog.get_mut(&args.0);
		if let Some(x) = var { x.string = args.1; }
		else { prog.insert(args.0, Variable { num: 0.0, string: args.1 }); }
		Ok(())
	}

	pub fn getStrFN(_: &Lua, args: String) -> Result<String, Error>
	{
		let prog = &mut Window::getWorld().prog;
		let var = prog.get(&args);
		if var.is_none() { return Ok(String::new()); }
		Ok(var.unwrap().string.clone())
	}

	pub fn hitboxFN(_: &Lua, id: usize) -> Result<(f32, f32, f32, f32), Error>
	{
		let h = Window::getWorld().ents[id].getRB().getHitbox();
		Ok((h.x, h.y, h.z, h.w))
	}
}