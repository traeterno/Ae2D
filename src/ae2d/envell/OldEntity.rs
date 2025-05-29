use mlua::{Error, Function, Lua, StdLib};
use spex::xml::Element;
use wrapped2d::b2;

use crate::ae2d::{Assets, Camera::Drawable, Programmable::{Programmable, Variable}, Transformable::Transformable2D, Window::Window};

use super::{AnimatedSprite::AnimatedSprite, World};

pub struct Physics
{
	pub body: b2::BodyHandle,
	pub hitboxes: Vec<(String, Vec<b2::Vec2>)>,
	pub scaleHitbox: bool,
	pub touchingGround: bool,
	pub touchingWall: bool,
	pub group: i16
}

pub struct Entity
{
	spr: AnimatedSprite,
	name: String,
	script: Lua,
	pub prog: Programmable,
	init: bool,
	pub physics: Physics,
	pub req: Option<[f32; 4]>,
	active: bool,
	visible: bool,
	pub destroyed: bool
}

impl Entity
{
	pub fn parse(node: &Element) -> Self
	{
		let mut prog = Programmable::new();
		let mut bodyDef = b2::BodyDef::new();
		let mut vertices: Vec<(String, Vec<b2::Vec2>)> = vec![];
		let mut scaleHitbox = true;
		let mut b2Name = String::new();
		let mut group = 0;

		for vars in node.elements()
		{
			let name = vars.name().local_part();
			if name == "physics"
			{
				bodyDef.body_type = match vars.att_opt("type").unwrap_or("")
				{
					"dynamic" => b2::BodyType::Dynamic,
					"kinematic" => b2::BodyType::Kinematic,
					_ => b2::BodyType::Static
				};
				scaleHitbox = vars.att_opt("scaleHitbox").unwrap_or("true").parse::<bool>().unwrap();
				bodyDef.fixed_rotation = vars.att_opt("fixedRotation").unwrap_or("false").parse::<bool>().unwrap();
				b2Name = vars.att_opt("name").unwrap_or("unknown").to_string();
				group = -vars.att_opt("group").unwrap_or("0").parse::<i16>().unwrap();

				for shape in vars.elements()
				{
					let mut v = vec![];
					for vertex in shape.elements()
					{
						if vertex.name().local_part() == "vertex"
						{
							v.push(b2::Vec2 {
									x: vertex.att_opt("x").unwrap_or("1").parse::<f32>().unwrap() / World::m2p,
									y: vertex.att_opt("y").unwrap_or("1").parse::<f32>().unwrap() / World::m2p
							});
						}
					}
					vertices.push((
						shape.att_opt("name").unwrap_or("").to_string(),
						v
					));
				}
				continue;
			}
			let txt = vars.text().unwrap_or("");
			let num = txt.parse::<f32>();
			prog.insert(
				name.to_string(),
				if num.is_ok() { Variable { num: num.unwrap(), string: String::new() } }
				else { Variable { num: 0.0, string: txt.to_string()} }
			);
		}

		let bodyHandle = Window::getWorld().world.create_body_with(&bodyDef, b2Name);
		let mut body = Window::getWorld().world.body_mut(bodyHandle);
		for (name, v) in &vertices
		{
			let mut fixdef = b2::FixtureDef::new();
			fixdef.density = 1.0;
			fixdef.filter.group_index = group;
			body.create_fixture_with(
				&b2::PolygonShape::new_with(
					&v.iter().map(|p| return b2::Vec2 {x: p.x, y: p.y}).collect::<Vec<b2::Vec2>>()
				),
				&mut fixdef,
				name.to_owned()
			);
		}

		let mut out = Self
		{
			spr: AnimatedSprite::new(),
			name: node.att_opt("name").unwrap_or("").to_string(),
			script: Lua::new(),
			prog,
			init: true,
			physics: Physics
			{
				body: bodyHandle,
				hitboxes: vertices,
				scaleHitbox,
				touchingGround: false,
				touchingWall: false,
				group
			},
			req: None,
			active: true,
			visible: true,
			destroyed: false
		};

		out.spr.loadAnimator(node.att_opt("anim").unwrap_or("").to_string());

		out.script.load_std_libs(StdLib::ALL_SAFE);
		out.initLua();
		Window::initLua(&out.script);
		out.script.load(
			Assets::readFile(node.att_opt("script")
				.unwrap_or("")
				.to_string())
				.unwrap()
		).exec();

		out
	}

	fn initLua(&mut self)
	{
		let table = self.script.create_table().unwrap();

		table.set("getNum", self.script.create_function(Entity::getNum).unwrap());
		table.set("getStr", self.script.create_function(Entity::getStr).unwrap());
		table.set("setNum", self.script.create_function(Entity::setNum).unwrap());
		table.set("setStr", self.script.create_function(Entity::setStr).unwrap());

		table.set("setPosition", self.script.create_function(Entity::setPosition).unwrap());
		table.set("getPosition", self.script.create_function(Entity::getPosition).unwrap());

		table.set("setScale", self.script.create_function(Entity::setScale).unwrap());
		table.set("scale", self.script.create_function(Entity::scale).unwrap());
		table.set("getScale", self.script.create_function(Entity::getScale).unwrap());

		table.set("setRotation", self.script.create_function(Entity::setRotation).unwrap());
		table.set("rotate", self.script.create_function(Entity::rotate).unwrap());
		table.set("getRotation", self.script.create_function(Entity::getRotation).unwrap());

		table.set("setOrigin", self.script.create_function(Entity::setOrigin).unwrap());
		table.set("getOrigin", self.script.create_function(Entity::getOrigin).unwrap());

		table.set("size", self.script.create_function(Entity::size).unwrap());
		table.set("bounds", self.script.create_function(Entity::bounds).unwrap());
		table.set("setAnimation", self.script.create_function(Entity::setAnimation).unwrap());
		table.set("name", self.script.create_function(Entity::name).unwrap());
		table.set("destroy", self.script.create_function(Entity::destroy).unwrap());

		table.set("setActive", self.script.create_function(Entity::setActive).unwrap());
		table.set("setVisible", self.script.create_function(Entity::setVisible).unwrap());

		table.set("setVelocity", self.script.create_function(Entity::setVelocity).unwrap());
		table.set("getVelocity", self.script.create_function(Entity::getVelocity).unwrap());
		table.set("applyForce", self.script.create_function(Entity::applyForce).unwrap());
		table.set("collisions", self.script.create_function(Entity::collisions).unwrap());

		self.script.globals().set("entity", table);
	}

	fn getNum(_: &Lua, name: String) -> Result<f64, Error> { unsafe { Ok(Window::getWorld().currentEnt.as_mut().unwrap().prog.get(&name).unwrap_or(&Variable::new()).num as f64) } }
	fn getStr(_: &Lua, name: String) -> Result<String, Error> { unsafe { Ok(Window::getWorld().currentEnt.as_mut().unwrap().prog.get(&name).unwrap_or(&Variable::new()).string.clone()) } }
	fn setNum(_: &Lua, x: (String, f64)) -> Result<(), Error> { unsafe { Window::getWorld().currentEnt.as_mut().unwrap().prog.insert(x.0, Variable { num: x.1 as f32, string: String::new() }); } Ok(()) }
	fn setStr(_: &Lua, x: (String, String)) -> Result<(), Error> { unsafe { Window::getWorld().currentEnt.as_mut().unwrap().prog.insert(x.0, Variable { num: 0.0, string: x.1 }); } Ok(()) }

	fn setPosition(_: &Lua, pos: (f64, f64)) -> Result<(), Error>
	{
		let ent = unsafe { Window::getWorld().currentEnt.as_mut().unwrap() };
		let mut body = Window::getWorld().world.body_mut(ent.physics.body);
		let angle = body.angle();
		body.set_transform(&b2::Vec2 {x: pos.0 as f32 / World::m2p, y: pos.1 as f32 / World::m2p}, angle);
		Ok(())
	}

	fn getPosition(_: &Lua, _: ()) -> Result<(f64, f64), Error>
	{
		let ent = unsafe { Window::getWorld().currentEnt.as_mut().unwrap() };
		let body = Window::getWorld().world.body(ent.physics.body);
		Ok(((body.position().x * World::m2p) as f64, (body.position().y * World::m2p) as f64))
	}

	fn setRotation(_: &Lua, angle: f64) -> Result<(), Error>
	{
		let ent = unsafe { Window::getWorld().currentEnt.as_mut().unwrap() };
		let mut body = Window::getWorld().world.body_mut(ent.physics.body);
		let pos = body.position().clone();
		body.set_transform(&pos, angle.to_radians() as f32);
		Ok(())
	}
	
	fn rotate(_: &Lua, angle: f64) -> Result<(), Error>
	{
		let ent = unsafe { Window::getWorld().currentEnt.as_mut().unwrap() };
		let mut body = Window::getWorld().world.body_mut(ent.physics.body);
		let pos = body.position().clone();
		let base = body.angle();
		body.set_transform(&pos, base + angle.to_radians() as f32);
		Ok(())
	}

	fn getRotation(_: &Lua, _: ()) -> Result<f64, Error>
	{
		let ent = unsafe { Window::getWorld().currentEnt.as_mut().unwrap() };
		let body = Window::getWorld().world.body(ent.physics.body);
		Ok(body.angle().to_degrees() as f64)
	}

	fn setOrigin(_: &Lua, origin: (f64, f64)) -> Result<(), Error> { unsafe { Window::getWorld().currentEnt.as_mut().unwrap().getTransform().setOrigin(glam::vec2(origin.0 as f32, origin.1 as f32)); } Ok(()) }
	fn getOrigin(_: &Lua, _: ()) -> Result<(f64, f64), Error> { let origin = unsafe { Window::getWorld().currentEnt.as_mut().unwrap().getTransform().getOrigin() }; Ok((origin.x as f64, origin.y as f64)) }

	fn size(_: &Lua, _: ()) -> Result<(f64, f64), Error> { let size = unsafe { Window::getWorld().currentEnt.as_mut().unwrap().spr.getAnimator().getFrameSize() }; Ok((size.x as f64, size.y as f64)) }
	fn bounds(_: &Lua, _: ()) -> Result<(f64, f64, f64, f64), Error> { let bounds = unsafe { Window::getWorld().currentEnt.as_mut().unwrap().spr.getBounds() }; Ok((bounds.left() as f64, bounds.top() as f64, bounds.width() as f64, bounds.height() as f64)) }
	fn setAnimation(_: &Lua, anim: String) -> Result<(), Error> { unsafe { Window::getWorld().currentEnt.as_mut().unwrap().spr.setAnimation(anim); } Ok(())}
	fn name(_: &Lua, _: ()) -> Result<String, Error> { unsafe { Ok(Window::getWorld().currentEnt.as_mut().unwrap().name.clone()) } }

	fn setActive(_: &Lua, x: bool) -> Result<(), Error> { unsafe { Window::getWorld().currentEnt.as_mut().unwrap().active = x; } Ok(()) }
	fn setVisible(_: &Lua, x: bool) -> Result<(), Error> { unsafe { Window::getWorld().currentEnt.as_mut().unwrap().visible = x; } Ok(()) }

	fn setScale(_: &Lua, scale: (f64, f64)) -> Result<(), Error>
	{
		let ent = unsafe { Window::getWorld().currentEnt.as_mut().unwrap() };
		ent.getTransform().setScale(glam::vec2(scale.0 as f32, scale.1 as f32));
		if ent.physics.scaleHitbox && ent.physics.hitboxes.len() > 0
		{
			let scale = ent.getTransform().getScale();
			let mut body = Window::getWorld().world.body_mut(ent.physics.body);
			let mut fixtures = vec![];
			for (fix, _) in body.fixtures() { fixtures.push(fix); }
			for fix in fixtures { body.destroy_fixture(fix); }
			for (name, vertices) in &ent.physics.hitboxes
			{
				let mut fixdef = b2::FixtureDef::new();
				fixdef.density = 1.0;
				fixdef.filter.group_index = ent.physics.group;
				body.create_fixture_with(
					&b2::PolygonShape::new_with(
						&vertices.iter().map(|p| return b2::Vec2 {x: p.x * scale.x, y: p.y * scale.y}).collect::<Vec<b2::Vec2>>()
					),
					&mut fixdef,
					name.to_owned()
				);
			}
		}
		Ok(())
	}
	fn scale(_: &Lua, scale: (f64, f64)) -> Result<(), Error>
	{
		let ent = unsafe { Window::getWorld().currentEnt.as_mut().unwrap() };
		ent.getTransform().scale(glam::vec2(scale.0 as f32, scale.1 as f32));
		if ent.physics.scaleHitbox && ent.physics.hitboxes.len() > 0
		{
			let scale = ent.getTransform().getScale();
			let mut body = Window::getWorld().world.body_mut(ent.physics.body);
			for (fix, _) in body.fixtures()
			{
				Window::getWorld().world.body_mut(ent.physics.body).destroy_fixture(fix);
			}
			for (name, vertices) in &ent.physics.hitboxes
			{
				let mut fixdef = b2::FixtureDef::new();
				fixdef.density = 1.0;
				fixdef.filter.group_index = ent.physics.group;
				body.create_fixture_with(
					&b2::PolygonShape::new_with(
						&vertices.iter().map(|p| return b2::Vec2 {x: p.x * scale.x, y: p.y * scale.y}).collect::<Vec<b2::Vec2>>()
					),
					&mut fixdef,
					name.to_owned()
				);
			}
		}
		Ok(())
	}
	fn getScale(_: &Lua, _: ()) -> Result<(f64, f64), Error> { let scale = unsafe { Window::getWorld().currentEnt.as_mut().unwrap().getTransform().getScale() }; Ok((scale.x as f64, scale.y as f64)) }

	fn setVelocity(_: &Lua, vel: (f64, f64)) -> Result<(), Error>
	{
		let ent = unsafe { Window::getWorld().currentEnt.as_mut().unwrap() };
		let mut body = Window::getWorld().world.body_mut(ent.physics.body);
		body.set_linear_velocity(&b2::Vec2 { x: vel.0 as f32 / World::m2p, y: vel.1 as f32 / World::m2p });
		Ok(())
	}

	fn getVelocity(_: &Lua, _: ()) -> Result<(f64, f64), Error>
	{
		let ent = unsafe { Window::getWorld().currentEnt.as_mut().unwrap() };
		let body = Window::getWorld().world.body(ent.physics.body);
		Ok(((body.linear_velocity().x * World::m2p) as f64, (body.linear_velocity().y * World::m2p) as f64))
	}

	fn applyForce(_: &Lua, vel: (f64, f64)) -> Result<(), Error>
	{
		let ent = unsafe { Window::getWorld().currentEnt.as_mut().unwrap() };
		let mut body = Window::getWorld().world.body_mut(ent.physics.body);
		body.apply_force_to_center(&b2::Vec2 { x: vel.0 as f32 / World::m2p, y: vel.1 as f32 / World::m2p }, true);
		Ok(())
	}

	fn collisions(_: &Lua, _: ()) -> Result<(bool, bool), Error>
	{
		let ent = unsafe { Window::getWorld().currentEnt.as_mut().unwrap() };
		Ok((ent.physics.touchingGround, ent.physics.touchingWall))
	}

	fn destroy(_: &Lua, _: ()) -> Result<(), Error> { unsafe { Window::getWorld().currentEnt.as_mut().unwrap().destroyed = true; } Ok(()) }
	
	fn luaError(&mut self, res: Result<(), Error>)
	{
		if res.is_ok() { return; }
		println!("Entity: {}\n{:?}\n", self.name, res.unwrap_err());
	}

	pub fn update(&mut self)
	{
		if !self.active { return; }

		if self.init
		{
			self.init = false;
			self.luaError(self.script.globals().get::<Function>("Init").unwrap().call(()));
		}
		self.luaError(self.script.globals().get::<Function>("Update").unwrap().call(()));

		let body = Window::getWorld().world.body(self.physics.body);
		let ts = self.spr.getTransform();
		ts.setPosition(glam::vec2(body.position().x * World::m2p, body.position().y * World::m2p));
		ts.setRotation(body.angle().to_degrees());
	}

	pub fn getTransform(&mut self) -> &mut Transformable2D { self.spr.getTransform() }
	pub fn getName(&mut self) -> &String { &self.name }

	pub fn netPack(&mut self) -> Vec<u8>
	{
		let body = Window::getWorld().world.body(self.physics.body);
		[
			body.linear_velocity().x.to_le_bytes(),
			body.linear_velocity().y.to_le_bytes(),
			body.position().x.to_le_bytes(),
			body.position().y.to_le_bytes()
		].concat().to_vec()
	}

	pub fn request(&mut self, data: [f32; 4]) { self.req = Some(data); }
	pub fn handleRequest(&mut self)
	{
		if self.req.is_none() { return; }

		let mut body = Window::getWorld().world.body_mut(self.physics.body);
		let x = self.req.unwrap();
		let angle = body.angle();
		body.set_linear_velocity(&b2::Vec2 { x: x[0], y: x[1] });
		body.set_transform(
			&b2::Vec2 { x: x[2], y: x[3] },
			angle
		);
		self.req = None;
	}
}

impl Drawable for Entity { fn draw(&mut self) { if self.visible { self.spr.draw(); } } }