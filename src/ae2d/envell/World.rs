use std::ptr::null;

use mlua::{Function, IntoLuaMulti, Lua, StdLib, Value::{self}};
use wrapped2d::user_data::UserDataTypes;

use crate::ae2d::{Assets, Camera::Drawable, Programmable::{Programmable, Variable}};

use super::Entity::Entity;

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
	updateFN: Option<Function>
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
			updateFN: None
		};

		world.script.load_std_libs(StdLib::ALL_SAFE);
		world
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
			ent.update();
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

	pub fn execute(&mut self, func: String, args: impl IntoLuaMulti)
	{
		if let Ok(f) =
			self.script.globals().get::<Function>(func)
		{
			f.call::<Value>(args);
		}
	}
}