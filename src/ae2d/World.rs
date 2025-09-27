use std::collections::HashMap;

use mlua::Lua;

use crate::ae2d::{bind, Camera::Drawable, Entity::Entity, Programmable::Programmable};

pub struct World
{
	name: String,
	script: Lua,
	ents: Vec<Entity>,
	prog: Programmable,
	triggers: HashMap<String, (String, glam::Vec4)>,
	layers: Vec<(Vec<String>, Vec<String>)>,
	init: bool
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
			triggers: HashMap::new(),
			layers: vec![],
			init: true
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

		self.init = true;

		bind::window(&self.script);
		bind::network(&self.script);
		bind::world(&self.script);
	}

	pub fn update(&mut self)
	{
		if self.init
		{
			bind::execFunc(&self.script, "Init");
			self.init = false;
		}
		for l in &mut self.layers { *l = (vec![], vec![]); }
		bind::execFunc(&self.script, "Update");
		for mut i in 0..self.ents.len() as i16
		{
			let (layer, opaque) = self.ents[i as usize].update();
			if layer == 255 { i -= 1; self.ents.remove((i + 1) as usize); continue; }
			if opaque
			{
				self.layers[layer as usize].0.push(self.ents[i as usize].getID());
			}
			else
			{
				self.layers[layer as usize].1.push(self.ents[i as usize].getID());
			}
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

	pub fn setLayersCount(&mut self, layers: u8)
	{
		self.layers.resize(layers as usize, (vec![], vec![]));
	}
}

impl Drawable for World
{
	fn draw(&mut self)
	{
		for layer in 0..self.layers.len()
		{
			unsafe
			{
				gl::Enable(gl::STENCIL_TEST);
				gl::Clear(gl::STENCIL_BUFFER_BIT);
				gl::Disable(gl::BLEND);
			}
			let opaque = self.layers[layer].0.len();
			for i in 0..opaque
			{
				self.getEntity(self.layers[layer].0[opaque - 1 - i].clone()).draw();
			}
			unsafe
			{
				gl::Enable(gl::BLEND);
				gl::Disable(gl::STENCIL_TEST);
			}
			let transparent = self.layers[layer].1.len();
			for i in 0..transparent
			{
				self.getEntity(self.layers[layer].1[i].clone()).draw();
			}
		}
		unsafe
		{
			gl::Finish();
			gl::Enable(gl::BLEND);
		}
	}
}