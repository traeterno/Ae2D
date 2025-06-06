use mlua::{Error, Function, Lua, Number, Table, Value};
use wrapped2d::b2;

use crate::ae2d::{Assets, Camera::Drawable, Programmable::{Programmable, Variable}, Window::Window};

use super::{AnimatedSprite::AnimatedSprite, World::{self}};

#[derive(Clone, Debug)]
pub struct Entity
{
	id: String,
	name: String,
	prog: Programmable,
	group: String,
	script: Lua,
	drawable: Vec<AnimatedSprite>,
	friendly: String,
	hostile: String,
	funcs: Option<(Function, Function)>,
	physics: Option<b2::BodyHandle>
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
			drawable: vec![],
			friendly: String::new(),
			hostile: String::new(),
			funcs: None,
			physics: None
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
			if name == "friendly"
			{
				ent.friendly = element.text().unwrap_or("").to_string();
			}
			if name == "hostile"
			{
				ent.hostile = element.text().unwrap_or("").to_string();
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
		World::World::initLua(&self.script);

		let update: Function;
		let draw: Function;

		match self.script.globals().get::<Function>("Init")
		{
			Ok(func) => { func.call::<Value>(()); }
			Err(_) => { panic!("Entity: {}\n'Init' function not found", self.id); }
		}
		match self.script.globals().get::<Function>("Update")
		{
			Ok(f) => { update = f; }
			Err(_) => { panic!("Entity: {}\n'Update' function not found", self.id); }
		}
		match self.script.globals().get::<Function>("Draw")
		{
			Ok(f) => { draw = f; }
			Err(_) => { panic!("Entity: {}\n'Draw' function not found", self.id); }
		}

		self.funcs = Some((update, draw));
	}

	pub fn update(&mut self)
	{
		self.funcs.as_mut().unwrap().0.call::<Value>(());
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

		let table = self.script.create_table().unwrap();

		table.set("init", self.script.create_function(Entity::physics_init).unwrap());
		table.set("setBodyType", self.script.create_function(Entity::physics_setBodyType).unwrap());
		table.set("setHitbox", self.script.create_function(Entity::physics_setHitbox).unwrap());
		table.set("getTransform", self.script.create_function(Entity::physics_getTransform).unwrap());
		table.set("setTransform", self.script.create_function(Entity::physics_setTransform).unwrap());
		table.set("getVelocity", self.script.create_function(Entity::physics_getVelocity).unwrap());
		table.set("setVelocity", self.script.create_function(Entity::physics_setVelocity).unwrap());
		table.set("applyForce", self.script.create_function(Entity::physics_applyForce).unwrap());

		self.script.globals().set("physics", table);
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
	
	pub fn getID(&mut self) -> &String { &self.id }
	pub fn getProgrammable(&mut self) -> &mut Programmable { &mut self.prog }

	pub fn physics_init(_: &Lua, data: Table) -> Result<(), Error>
	{
		let mut bd = b2::BodyDef::new();
		for info in data.pairs::<String, Value>()
		{
			if info.is_err() { continue; }
			let (name, value) = info.unwrap();
			if name == "lockRotation" { bd.fixed_rotation = value.as_boolean().unwrap_or(false); }
		}
		let ent = Window::getWorld().getCurrentEntity();
		ent.physics = Some(
			Window::getWorld().getB2D().create_body_with(&bd, ent.id.clone())
		);
		Ok(())
	}

	pub fn physics_setBodyType(_: &Lua, data: String) -> Result<(), Error>
	{
		let ent = Window::getWorld().getCurrentEntity();
		if ent.physics.is_none() { return Ok(()); }
		let mut body = Window::getWorld().getB2D().body_mut(ent.physics.unwrap());
		body.set_body_type(
			if data == "static" { b2::BodyType::Static }
			else if data == "kinematic" { b2::BodyType::Kinematic }
			else { b2::BodyType::Dynamic }
		);
		Ok(())
	}

	pub fn physics_setHitbox(_: &Lua, data: Table) -> Result<(), Error>
	{
		let ent = Window::getWorld().getCurrentEntity();
		if ent.physics.is_none() { return Ok(()); }
		let mut body = Window::getWorld().getB2D().body_mut(ent.physics.unwrap());
		
		let mut list = vec![];
		for handle in body.fixtures()
		{
			list.push(handle.0);
		}

		for i in list
		{
			body.destroy_fixture(i);
		}

		let mut fd = b2::FixtureDef::new();
		fd.density = 1.0;

		for hitbox in data.pairs::<String, Value>()
		{
			if hitbox.is_err() { continue; }
			let (name, data) = hitbox.unwrap();

			if name == "Friction" { fd.friction = data.as_f32().unwrap_or(0.0); continue; }

			let data = data.as_table().unwrap();

			let mut shape = vec![];

			for point in data.pairs::<Value, Table>()
			{
				if point.is_err() { continue; }
				let (_, point) = point.unwrap();
				shape.push(b2::Vec2 {
					x: (point.raw_get::<Number>(1).unwrap() as f32) / World::m2p,
					y: (point.raw_get::<Number>(2).unwrap() as f32) / World::m2p
				});
			}

			body.create_fixture_with(
				&b2::PolygonShape::new_with(&shape),
				&mut fd,
				name
			);
		}
		
		Ok(())
	}

	pub fn physics_getTransform(_: &Lua, _: ()) -> Result<(Number, Number, Number), Error>
	{
		let ent = Window::getWorld().getCurrentEntity();
		if ent.physics.is_none() { return Ok((0.0, 0.0, 0.0)); }
		let body = Window::getWorld().getB2D().body(ent.physics.unwrap());
		Ok((
			(body.position().x * World::m2p) as f64,
			(body.position().y * World::m2p) as f64,
			body.angle().to_degrees() as f64
		))
	}

	pub fn physics_setTransform(_: &Lua, data: (f32, f32, f32)) -> Result<(), Error>
	{
		let ent = Window::getWorld().getCurrentEntity();
		if ent.physics.is_none() { return Ok(()); }
		let mut body = Window::getWorld().getB2D().body_mut(ent.physics.unwrap());

		body.set_transform(
			&b2::Vec2 {
				x: data.0 / World::m2p,
				y: data.1 / World::m2p
			},
			data.2.to_radians()
		);
		Ok(())
	}

	pub fn physics_setVelocity(_: &Lua, data: (f32, f32)) -> Result<(), Error>
	{
		let ent = Window::getWorld().getCurrentEntity();
		if ent.physics.is_none() { return Ok(()); }
		let mut body = Window::getWorld().getB2D().body_mut(ent.physics.unwrap());
		body.set_linear_velocity(&b2::Vec2 {
			x: data.0 / World::m2p,
			y: data.1 / World::m2p
		});
		Ok(())
	}

	pub fn physics_getVelocity(_: &Lua, _: ()) -> Result<(f32, f32), Error>
	{
		let ent = Window::getWorld().getCurrentEntity();
		if ent.physics.is_none() { return Ok((0.0, 0.0)); }
		let body = Window::getWorld().getB2D().body(ent.physics.unwrap());
		Ok((
			body.linear_velocity().x * World::m2p,
			body.linear_velocity().y * World::m2p
		))
	}

	pub fn physics_applyForce(_: &Lua, f: (f32, f32)) -> Result<(), Error>
	{
		let ent = Window::getWorld().getCurrentEntity();
		if ent.physics.is_none() { return Ok(()); }
		let mut body = Window::getWorld().getB2D().body_mut(ent.physics.unwrap());
		body.apply_force_to_center(&b2::Vec2 {
			x: f.0, y: f.1
		}, true);
		Ok(())
	}
}

impl Drawable for Entity
{
	fn draw(&mut self)
	{
		Window::getWorld().setCurrentEntity(self);
		self.funcs.as_mut().unwrap().1.call::<Value>(());
	}
}