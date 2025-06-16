use mlua::{Error, Function, Lua, Value};

use crate::ae2d::{Assets, Camera::Drawable, Programmable::{Programmable, Variable}, Window::Window};

use super::{AnimatedSprite::AnimatedSprite, Physics::Rigidbody, World::World};

#[derive(Clone, Debug)]
pub struct Entity
{
	id: String,
	name: String,
	prog: Programmable,
	group: String,
	script: Lua,
	rb: Rigidbody,
	drawable: Vec<AnimatedSprite>,
	prePhysicsFN: Option<Function>,
	onCollidedFN: Option<Function>,
	midPhysicsFN: Option<Function>,
	postPhysicsFN: Option<Function>,
	drawFN: Option<Function>
}

impl Entity
{
	pub fn new() -> Self
	{
		Self
		{
			id: String::new(),
			name: String::new(),
			prog: Programmable::new(),
			group: String::new(),
			script: Lua::new(),
			rb: Rigidbody::new(),
			drawable: vec![],
			prePhysicsFN: None,
			onCollidedFN: None,
			midPhysicsFN: None,
			postPhysicsFN: None,
			drawFN: None
		}
	}
	pub fn load(path: String) -> Self
	{
		let mut ent = Self::new();

		let doc = Assets::readXML(path);
		if doc.is_none() { return Self::new(); }
		let doc = doc.unwrap();

		for element in doc.elements()
		{
			let name = element.name().local_part();
			if name == "name"
			{
				ent.name = element.text().unwrap_or("").to_string();
			}
			if name == "group"
			{
				ent.group = element.text().unwrap_or("").to_string();
			}
			if name == "script"
			{
				ent.script.load(
					Assets::readFile(
						element.text().unwrap_or("").to_string()
					).unwrap_or(String::new()).to_string()
				).exec();
			}
		}

		ent
	}

	pub fn init(&mut self, id: String, prog: Programmable)
	{
		self.id = id;
		
		for (key, value) in prog
		{
			self.prog.insert(key, value);
		}

		self.initLua();
		AnimatedSprite::initLua(&self.script);
		Window::initLua(&self.script);
		World::initLua(&self.script);
		Rigidbody::initLua(&self.script);

		match self.script.globals().get::<Function>("Init")
		{
			Ok(func) => { func.call::<Value>(()); }
			Err(_) => { println!("Entity: {}\n'Init' function not found", self.id); }
		}
		match self.script.globals().get::<Function>("PrePhysics")
		{
			Ok(f) => { self.prePhysicsFN = Some(f); }
			Err(_) => { println!("Entity: {}\n'PrePhysics' function not found", self.id); }
		}
		match self.script.globals().get::<Function>("OnCollided")
		{
			Ok(f) => { self.onCollidedFN = Some(f); }
			Err(_) => { println!("Entity: {}\n'OnCollided' function not found", self.id); }
		}
		match self.script.globals().get::<Function>("PostPhysics")
		{
			Ok(f) => { self.postPhysicsFN = Some(f); }
			Err(_) => { println!("Entity: {}\n'PostPhysics' function not found", self.id); }
		}
		match self.script.globals().get::<Function>("MidPhysics")
		{
			Ok(f) => { self.midPhysicsFN = Some(f); }
			Err(_) => { println!("Entity: {}\n'MidPhysics' function not found", self.id); }
		}
		match self.script.globals().get::<Function>("Draw")
		{
			Ok(f) => { self.drawFN = Some(f); }
			Err(_) => { println!("Entity: {}\n'Draw' function not found", self.id); }
		}
	}

	fn initLua(&mut self)
	{
		let table = self.script.create_table().unwrap();

		table.set("getNum", self.script.create_function(Entity::getNumFN).unwrap());
		table.set("getStr", self.script.create_function(Entity::getStrFN).unwrap());
		table.set("setNum", self.script.create_function(Entity::setNumFN).unwrap());
		table.set("setStr", self.script.create_function(Entity::setStrFN).unwrap());
		table.set("getName", self.script.create_function(Entity::getNameFN).unwrap());
		table.set("setName", self.script.create_function(Entity::setNameFN).unwrap());
		table.set("id", self.script.create_function(Entity::idFN).unwrap());
		table.set("createSprite", self.script.create_function(Entity::createSprite).unwrap());

		self.script.globals().set("entity", table);
	}

	pub fn setNumFN(_: &Lua, args: (String, f32)) -> Result<(), Error>
	{
		let prog = Window::getWorld().getCurrentEntity().getProgrammable();
		let var = prog.get_mut(&args.0);
		if let Some(x) = var { x.num = args.1; }
		else { prog.insert(args.0, Variable { num: args.1, string: String::new() }); }
		Ok(())
	}

	pub fn getNumFN(_: &Lua, args: String) -> Result<f64, Error>
	{
		let prog = Window::getWorld().getCurrentEntity().getProgrammable();
		let var = prog.get(&args);
		if var.is_none() { return Ok(0.0); }
		Ok(var.unwrap().num as f64)
	}

	pub fn setStrFN(_: &Lua, args: (String, String)) -> Result<(), Error>
	{
		let prog = Window::getWorld().getCurrentEntity().getProgrammable();
		let var = prog.get_mut(&args.0);
		if let Some(x) = var { x.string = args.1; }
		else { prog.insert(args.0, Variable { num: 0.0, string: args.1 }); }
		Ok(())
	}

	pub fn getStrFN(_: &Lua, args: String) -> Result<String, Error>
	{
		let prog = Window::getWorld().getCurrentEntity().getProgrammable();
		let var = prog.get(&args);
		if var.is_none() { return Ok(String::new()); }
		Ok(var.unwrap().string.clone())
	}

	pub fn setNameFN(_: &Lua, name: String) -> Result<(), Error>
	{
		Window::getWorld().getCurrentEntity().name = name;
		Ok(())
	}

	pub fn getNameFN(_: &Lua, _: ()) -> Result<String, Error>
	{
		Ok(Window::getWorld().getCurrentEntity().name.clone())
	}

	pub fn idFN(_: &Lua, _: ()) -> Result<String, Error>
	{
		Ok(Window::getWorld().getCurrentEntity().getID().to_string())
	}

	pub fn createSprite(_: &Lua, path: String) -> Result<i32, Error>
	{
		let ent = Window::getWorld().getCurrentEntity();
		let mut spr = AnimatedSprite::new();
		spr.loadAnimator(path);
		ent.drawable.push(spr);
		Ok((ent.drawable.len() - 1) as i32)
	}

	pub fn getSprite(&mut self, id: usize) -> &mut AnimatedSprite
	{
		self.drawable.get_mut(id).unwrap()
	}

	pub fn prePhysics(&mut self)
	{
		Window::getWorld().setCurrentEntity(self);
		if let Some(f) = &self.prePhysicsFN
		{
			f.call::<Value>(());
		}
	}

	pub fn onCollided(&mut self, dir: u8, name: String, id: String, h: glam::Vec4)
	{
		Window::getWorld().setCurrentEntity(self);
		if let Some(f) = &self.onCollidedFN
		{
			f.call::<Value>((dir, name, id, h.x, h.y, h.z, h.w));
		}
	}

	pub fn midPhysics(&mut self)
	{
		Window::getWorld().setCurrentEntity(self);
		if let Some(f) = &self.midPhysicsFN
		{
			f.call::<Value>(());
		}
	}

	pub fn postPhysics(&mut self)
	{
		Window::getWorld().setCurrentEntity(self);
		if let Some(f) = &self.postPhysicsFN
		{
			f.call::<Value>(());
		}
	}

	pub fn execute(&mut self, cmd: String)
	{
		self.script.load(cmd).exec();
	}
	
	pub fn getRB(&mut self) -> &mut Rigidbody { &mut self.rb }
	pub fn getID(&self) -> &String { &self.id }
	pub fn getName(&self) -> &String { &self.name }
	pub fn getProgrammable(&mut self) -> &mut Programmable { &mut self.prog }
}

impl Drawable for Entity
{
	fn draw(&mut self)
	{
		Window::getWorld().setCurrentEntity(self);
		if let Some(f) = &self.drawFN
		{
			f.call::<Value>(());
		}
	}
}