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

pub fn line(p1: glam::Vec2, p2: glam::Vec2, c1: glam::Vec4, c2: glam::Vec4)
{
	// unsafe
	// {
	// 	let mut vbo = 0;
	// 	let mut vao = 0;
	// 	gl::GenBuffers(1, &mut vbo);
	// 	gl::GenVertexArrays(1, &mut vao);
	// 	Window::getCamera().bindVAO(vao);
	// 	gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
	// 	gl::EnableVertexAttribArray(0);	
	// 	gl::VertexAttribPointer(
	// 		0, 2, gl::FLOAT,
	// 		gl::FALSE,
	// 		(6 * size_of::<f32>()) as _,
	// 		0 as _
	// 	);

	// 	gl::EnableVertexAttribArray(1);
	// 	gl::VertexAttribPointer(
	// 		1, 4, gl::FLOAT,
	// 		gl::FALSE,
	// 		(6 * size_of::<f32>()) as _,
	// 		(2 * size_of::<f32>()) as _
	// 	);

	// 	let data = [
	// 		p1.x, p1.y, c1.x, c1.y, c1.z, c1.w,
	// 		p2.x, p2.y, c2.x, c2.y, c2.z, c2.w,
	// 	];

	// 	gl::BufferData(gl::ARRAY_BUFFER,
	// 		(12 * size_of::<f32>()) as _,
	// 		data.as_ptr() as _,
	// 		gl::STATIC_DRAW
	// 	);

	// 	let s = Window::getShader(String::from("shape"));
	// 	s.activate();
	// 	s.setMat4("model", glam::Mat4::IDENTITY);
	// 	s.setVec2("size", glam::Vec2::ONE);
	// 	s.setVec4("clr", glam::Vec4::ONE);
	// 	gl::DrawArrays(gl::LINES, 0, 2);
	// 	gl::BindVertexArray(0);
	// 	gl::BindBuffer(gl::ARRAY_BUFFER, 0);
	// 	gl::DeleteBuffers(1, &mut vbo);
	// 	gl::DeleteVertexArrays(1, &mut vao);
	// }
}