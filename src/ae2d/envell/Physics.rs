use glam::Vec4Swizzles;
use mlua::{Error, Lua, Table};

use crate::ae2d::{Shapes::Rectangle, Window::Window};

#[derive(Clone, Debug)]
pub struct Rigidbody
{
	pos: glam::Vec2,
	size: glam::Vec2,
	origin: glam::Vec2
}

impl Rigidbody
{
	pub fn new() -> Self
	{
		Self
		{
			pos: glam::Vec2::ZERO,
			size: glam::Vec2::ZERO,
			origin: glam::Vec2::ZERO
		}
	}

	pub fn getHitbox(&self) -> glam::Vec4
	{
		glam::vec4(
			self.pos.x - self.origin.x,
			self.pos.y - self.origin.y,
			self.size.x, self.size.y
		)
	}

	pub fn checkCollision(&mut self, h2: glam::Vec4) -> bool
	{
		let h1 = self.getHitbox();

		let il = f32::max(h1.x, h2.x);
		let it = f32::max(h1.y, h2.y);
		let ir = f32::min(h1.x + h1.z, h2.x + h2.z);
		let ib = f32::min(h1.y + h1.w, h2.y + h2.w);

		(il < ir) && (it < ib)
	}

	pub fn initLua(script: &Lua)
	{
		let t = script.create_table().unwrap();

		t.set("setHitbox", script.create_function(Rigidbody::setHitbox).unwrap());
		t.set("setPosition", script.create_function(Rigidbody::setPosition).unwrap());
		t.set("draw", script.create_function(Rigidbody::draw).unwrap());

		script.globals().raw_set("physics", t);
	}

	pub fn setHitbox(_: &Lua, data: Table) -> Result<(), Error>
	{
		let e = Window::getWorld().getCurrentEntity();
		let name = e.getName().clone();
		let rb = e.getRB();

		let s = data.raw_get::<Table>("Size");
		if s.is_err() { println!("Entity {name}: size of hitbox not found"); return Ok(()); }
		let s = s.unwrap();

		let o = data.raw_get::<Table>("Origin");
		if o.is_err() { println!("Entity {name}: origin of hitbox not found"); return Ok(()); }
		let o = o.unwrap();
		
		rb.size = glam::vec2(
			s.raw_get::<f32>(1).unwrap_or(0.0),
			s.raw_get::<f32>(2).unwrap_or(0.0)
		);

		rb.origin = glam::vec2(
			o.raw_get::<f32>(1).unwrap_or(0.0),
			o.raw_get::<f32>(2).unwrap_or(0.0)
		);
		
		Ok(())
	}

	pub fn setPosition(_: &Lua, pos: (f32, f32)) -> Result<(), Error>
	{
		let rb = Window::getWorld().getCurrentEntity().getRB();
		rb.pos = glam::vec2(pos.0, pos.1);
		Ok(())
	}

	pub fn draw(_: &Lua, clr: (u8, u8, u8, u8)) -> Result<(), Error>
	{
		let mut obj = Rectangle::new();
		let h = Window::getWorld().getCurrentEntity().getRB().getHitbox();
		obj.setColor(glam::vec4(
			clr.0 as f32 / 255.0,
			clr.1 as f32 / 255.0,
			clr.2 as f32 / 255.0,
			clr.3 as f32 / 255.0
		));
		obj.setSize(h.zw());
		obj.getTransform().setPosition(h.xy());
		
		Window::getCamera().draw(&mut obj);
		Ok(())
	}
}