use mlua::{Error, Function, Lua, Value};

use crate::ae2d::{Assets, Camera::Drawable, Programmable::{Programmable, Variable}, Window::Window};

use super::AnimatedSprite::AnimatedSprite;

#[derive(Clone, Debug)]
struct CustomFuncs
{
	init: Option<Function>,
	update: Option<Function>,
	draw: Option<Function>,
}

impl Default for CustomFuncs { fn default() -> Self { Self { init: None, update: None, draw: None } } }

#[derive(Clone, Debug)]
pub struct Entity
{
	id: String,
	name: String,
	prog: Programmable,
	group: String,
	script: Lua,
	sprite: AnimatedSprite,
	friendly: String,
	hostile: String,
	cf: CustomFuncs
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
			sprite: AnimatedSprite::new(),
			friendly: String::new(),
			hostile: String::new(),
			cf: CustomFuncs::default()
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
			if name == "animation"
			{
				ent.sprite.loadAnimator(
					element.text().unwrap_or("").to_string()
				);
			}
			if name == "friendly"
			{
				ent.friendly = element.text().unwrap_or("").to_string();
			}
			if name == "hostile"
			{
				ent.hostile = element.text().unwrap_or("").to_string();
			}
		}

		ent.sprite.initLua(&mut ent.script);

		ent.cf = CustomFuncs
		{
			init: if let Ok(func) = ent.script.globals().get::<Function>("Init")
				{ Some(func) } else { None },
			update: if let Ok(func) = ent.script.globals().get::<Function>("Update")
				{ Some(func) } else { None },
			draw: if let Ok(func) = ent.script.globals().get::<Function>("Draw")
				{ Some(func) } else { None }
		};

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

		Window::initLua(&self.script);
		
		if let Some(func) = &self.cf.init { func.call::<Value>(()); }
	}

	pub fn update(&mut self)
	{
		if let Some(func) = &self.cf.update { func.call::<Value>(()); }
	}

	fn initLua(&mut self)
	{
		let table = self.script.create_table().unwrap();

		table.set("getNum", self.script.create_function(Entity::getNumFN).unwrap());
		table.set("getStr", self.script.create_function(Entity::getStrFN).unwrap());
		table.set("setNum", self.script.create_function(Entity::setNumFN).unwrap());
		table.set("setStr", self.script.create_function(Entity::setStrFN).unwrap());
		table.set("name", self.script.create_function(Entity::nameFN).unwrap());
		table.set("id", self.script.create_function(Entity::idFN).unwrap());

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

	pub fn nameFN(_: &Lua, _: ()) -> Result<String, Error>
	{
		Ok(Window::getWorld().getCurrentEntity().getName().to_string())
	}

	pub fn idFN(_: &Lua, _: ()) -> Result<String, Error>
	{
		Ok(Window::getWorld().getCurrentEntity().getID().to_string())
	}

	pub fn getName(&mut self) -> &String { &self.name }
	pub fn getID(&mut self) -> &String { &self.id }
	pub fn getSprite(&mut self) -> &mut AnimatedSprite { &mut self.sprite }
	pub fn getProgrammable(&mut self) -> &mut Programmable { &mut self.prog }
}

impl Drawable for Entity
{
	fn draw(&mut self)
	{
		if let Some(func) = &self.cf.draw { func.call::<Value>(()); }
		else { self.sprite.draw(); }
	}
}