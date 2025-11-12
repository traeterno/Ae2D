use crate::ae2d::{Camera::Drawable, Transformable::Transformable2D, Window::Window};

pub struct Rectangle
{
	size: glam::Vec2,
	color: glam::Vec4,
	ts: Transformable2D
}

impl Rectangle
{
	pub fn new() -> Self
	{
		Self
		{
			size: glam::Vec2::ZERO,
			ts: Transformable2D::new(),
			color: glam::Vec4::ONE
		}
	}

	pub fn getTransform(&mut self) -> &mut Transformable2D { &mut self.ts }
	
	pub fn setColor(&mut self, clr: glam::Vec4) { self.color = clr; }
	pub fn getColor(&mut self) -> glam::Vec4 { self.color }
	
	pub fn setSize(&mut self, size: glam::Vec2) { self.size = size; }
	pub fn getSize(&mut self) -> glam::Vec2 { self.size }
}

impl Drawable for Rectangle
{
	fn draw(&mut self)
	{
		let shader = Window::getCamera()
			.activateShader(String::from("shape"));
		shader.setVec2("size", self.size);
		shader.setMat4("model", self.ts.getMatrix());
		shader.setVec4("clr", self.color);
		Window::getCamera().universalVAO();
		unsafe { gl::DrawArrays(gl::QUADS, 0, 4); }
	}
}