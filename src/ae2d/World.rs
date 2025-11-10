use std::collections::HashMap;

use mlua::Lua;

use crate::ae2d::{Camera::Drawable, Entity::Entity, Programmable::Programmable, Window::Window, bind};

pub struct World
{
	name: String,
	script: Lua,
	ents: HashMap<String, Entity>,
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
			ents: HashMap::new(),
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
		Window::getProfiler().restart();
		if self.init
		{
			bind::execFunc(&self.script, "Init");
			self.init = false;
		}
		for l in &mut self.layers
		{
			*l = (vec![], vec![]);
		}
		bind::execFunc(&self.script, "Update");
		for (_, ent) in &mut self.ents
		{
			let (layer, opaque) = ent.update();
			if layer == 255 { continue; }
			if opaque { self.layers[layer as usize].0.push(ent.getID()); }
			else { self.layers[layer as usize].1.push(ent.getID()); }
		}
		Window::getProfiler().save("worldUpdate".to_string());
	}

	pub fn getEntity(&mut self, id: String) -> &mut Entity
	{
		if let Some(e) = self.ents.get_mut(&id)
		{
			return e;
		}
		panic!("Entity '{id}' not found");
	}

	pub fn spawn(&mut self, id: String, path: String, vars: json::JsonValue)
	{
		self.ents.insert(id.clone(), Entity::load(id.clone(), path));
		self.ents.get_mut(&id).unwrap().init(vars);
	}

	pub fn kill(&mut self, id: String)
	{
		self.ents.remove(&id);
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
		Window::getProfiler().restart();
		for layer in 0..self.layers.len()
		{
			let opaque = self.layers[layer].0.len();
			if opaque > 0
			{
				unsafe
				{
					gl::Enable(gl::STENCIL_TEST);
					gl::Clear(gl::STENCIL_BUFFER_BIT);
					gl::Disable(gl::BLEND);
				}
				for i in 1..=opaque
				{
					self.getEntity(self.layers[layer].0[opaque - i].clone()).draw();
				}
			}
			let transparent = self.layers[layer].1.len();
			if transparent > 0
			{
				unsafe { gl::Enable(gl::BLEND); gl::Disable(gl::STENCIL_TEST); }
				for i in 0..transparent
				{
					self.getEntity(self.layers[layer].1[i].clone()).draw();
				}
			}
		}
		unsafe { gl::Enable(gl::BLEND); gl::Finish(); }
		Window::getProfiler().save("worldDraw".to_string());
	}
}