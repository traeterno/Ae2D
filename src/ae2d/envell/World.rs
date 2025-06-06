use std::ptr::null;

use mlua::{Error, Function, IntoLuaMulti, Lua, StdLib, Value::{self}};
use wrapped2d::{b2, user_data::UserDataTypes};

use crate::ae2d::{Assets, Camera::Drawable, Network::Network, Programmable::{Programmable, Variable}, Window::Window};

use super::{DebugDraw::DebugDraw, Entity::Entity};

pub const m2p: f32 = 64.0;

pub struct EntData;

impl UserDataTypes for EntData
{
	type BodyData = String;
	type FixtureData = String;
	type JointData = String;
}

pub struct World
{
	ents: Vec<Entity>,
	script: Lua,
	currentEnt: *mut Entity,
	updateFN: Option<Function>,
	postUpdateFN: Option<Function>,
	prog: Programmable,
	b2d: b2::World<EntData>,
	debugDraw: DebugDraw
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
			postUpdateFN: None,
			prog: Programmable::new(),
			b2d: b2::World::new(&b2::Vec2 { x: 0.0, y: 0.0 }),
			debugDraw: DebugDraw {}
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
		table.set("setGravity", script.create_function(World::setGravityFN).unwrap());
		script.globals().set("world", table);
	}

	pub fn load(&mut self, path: String)
	{
		self.ents.clear();

		let doc = Assets::readXML(path);
		if doc.is_none() { return; }
		let doc = doc.unwrap();

		self.script.load(Assets::readFile(
			doc.att_opt("script")
			.unwrap_or("")
			.to_string()
		).unwrap()).exec();

		for el in doc.elements()
		{
			if el.name().local_part() == "entity"
			{
				let mut ent = Entity::load(
					el.att_opt("path").unwrap_or("").to_string()
				);

				let mut prog = Programmable::new();
				for var in el.elements()
				{
					if var.name().local_part() == "var"
					{
						prog.insert(
							var.att_opt("name").unwrap_or("").to_string(),
							Variable
							{
								num: var.att_opt("num").unwrap_or("0").parse().unwrap(),
								string: var.att_opt("str").unwrap_or("").to_string()
							}
						);
					}
				}

				self.currentEnt = &mut ent;

				ent.init(
					el.att_opt("id").unwrap_or("").to_string(),
					prog
				);

				self.ents.push(ent);
			}
		}

		if let Ok(func) = self.script.globals().get::<Function>("Init")
		{
			func.call::<Value>(());
		}

		if let Ok(func) = self.script.globals().get::<Function>("Update")
		{
			self.updateFN = Some(func);
		}

		if let Ok(func) = self.script.globals().get::<Function>("PostUpdate")
		{
			self.postUpdateFN = Some(func);
		}
	}

	pub fn update(&mut self)
	{
		self.b2d.step(Window::getDeltaTime(), 12, 8);
		if let Some(func) = &self.updateFN
		{
			func.call::<Value>(());
		}

		for ent in &mut self.ents
		{
			self.currentEnt = ent;
			ent.update();
		}

		if let Some(func) = &self.postUpdateFN
		{
			func.call::<Value>(());
		}
	}

	pub fn render(&mut self)
	{
		// let cam = Window::getCamera();
		// cam.toggleCameraTransform(true);

		for ent in &mut self.ents
		{
			ent.draw();
		}

		self.b2d.draw_debug_data(&mut self.debugDraw, b2::DrawFlags::DRAW_SHAPE);
	}

	pub fn getCurrentEntity(&mut self) -> &mut Entity
	{
		unsafe { self.currentEnt.as_mut().unwrap() }
	}

	pub fn setCurrentEntity(&mut self, ent: &mut Entity)
	{
		self.currentEnt = ent;
	}

	pub fn execute(&mut self, func: String, args: impl IntoLuaMulti)
	{
		if let Ok(f) =
			self.script.globals().get::<Function>(func)
		{
			f.call::<Value>(args);
		}
	}

	pub fn getB2D(&mut self) -> &mut b2::World<EntData> { &mut self.b2d }

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

	pub fn setGravityFN(_: &Lua, g: (f32, f32)) -> Result<(), Error>
	{
		Window::getWorld().b2d.set_gravity(&b2::Vec2 { x: g.0, y: g.1 });
		Ok(())
	}
}