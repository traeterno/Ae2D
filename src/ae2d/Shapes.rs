use crate::ae2d::{Camera::Drawable, Transformable::Transformable2D, Window::Window};

pub struct Rectangle
{
	size: glam::Vec2,
	update: bool,
	vbo: u32,
	vao: u32,
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
			update: true,
			vao: 0,
			vbo: 0,
			ts: Transformable2D::new(),
			color: glam::Vec4::ONE
		}
	}

	pub fn getTransform(&mut self) -> &mut Transformable2D { &mut self.ts }
	
	pub fn setColor(&mut self, clr: glam::Vec4) { self.color = clr; }
	pub fn getColor(&mut self) -> glam::Vec4 { self.color }
	
	pub fn setSize(&mut self, size: glam::Vec2) { self.size = size; self.update = true; }
	pub fn getSize(&mut self) -> glam::Vec2 { self.size }

	pub fn reload(&mut self)
	{
		if !self.update { return; }
		self.update = false;

		unsafe
		{
			if self.vao == 0
			{
				gl::GenVertexArrays(1, &mut self.vao);
				gl::GenBuffers(1, &mut self.vbo);
	
				gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
				gl::BindVertexArray(self.vao);
	
				gl::EnableVertexAttribArray(0);
				gl::VertexAttribPointer(
					0, 2, gl::FLOAT, gl::FALSE,
					(2 * size_of::<f32>()) as _,
					0 as _
				)
			}

			let vertices = [
				0.0, 0.0,
				self.size.x, 0.0,
				self.size.x, self.size.y,
				0.0, self.size.y
			];

			gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
			gl::BufferData(
				gl::ARRAY_BUFFER,
				(8 * size_of::<f32>()) as _,
				vertices.as_ptr() as _,
				gl::DYNAMIC_DRAW
			);
		}
	}
}

impl Drawable for Rectangle
{
	fn draw(&mut self)
	{
		self.reload();

		let shader = Window::getCamera().getShapeShader();
		shader.activate();
		shader.setMat4("model", self.ts.getMatrix());
		shader.setVec4("clr", self.color);
		unsafe
		{
			gl::BindVertexArray(self.vao);
			gl::DrawArrays(gl::QUADS, 0, 4);
			gl::BindVertexArray(0);
		}
	}
}