use wrapped2d::b2::Draw;

use crate::ae2d::Window::Window;

use super::World;

pub struct DebugDraw;

impl Draw for DebugDraw
{
	fn draw_polygon(&mut self, vertices: &[wrapped2d::b2::Vec2], color: &wrapped2d::b2::Color)
	{

		let shader = Window::getCamera().getShapeShader();
		let clr = glam::vec4(color.r, color.g, color.b, 0.1);
		let mut vao = 0;
		let mut vbo = 0;

		shader.activate();
		shader.setVec4("clr", clr.to_array());
		shader.setMat4("model", &glam::Mat4::IDENTITY.to_cols_array());

		unsafe
		{
			gl::GenVertexArrays(1, &mut vao);
			gl::GenBuffers(1, &mut vbo);

			gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
			gl::BindVertexArray(vao);

			gl::EnableVertexAttribArray(0);
			gl::VertexAttribPointer(
				0, 2, gl::FLOAT, gl::FALSE,
				(2 * size_of::<f32>()) as _,
				0 as _
			);

			let mut v: Vec<f32> = vec![];
			for vertex in vertices
			{
				v.push(vertex.x * World::m2p);
				v.push(vertex.y * World::m2p);
			}

			gl::BufferData(
				gl::ARRAY_BUFFER,
				(v.len() * size_of::<f32>()) as _,
				v.as_ptr() as _,
				gl::DYNAMIC_DRAW
			);

			gl::DrawArrays(gl::TRIANGLE_FAN, 0, v.len() as _);

			gl::BindVertexArray(0);
			gl::BindBuffer(gl::ARRAY_BUFFER, 0);
			gl::DeleteVertexArrays(1, &mut vao);
			gl::DeleteBuffers(1, &mut vbo);
		}

		shader.setInt("layer", shader.getInt("layer") + 1);
	}

	fn draw_solid_polygon(&mut self, vertices: &[wrapped2d::b2::Vec2], color: &wrapped2d::b2::Color)
	{
		let shader = Window::getCamera().getShapeShader();
		let clr = glam::vec4(color.r, color.g, color.b, 0.1);
		let mut vao = 0;
		let mut vbo = 0;

		shader.activate();
		shader.setVec4("clr", clr.to_array());
		shader.setMat4("model", &glam::Mat4::IDENTITY.to_cols_array());

		unsafe
		{
			gl::GenVertexArrays(1, &mut vao);
			gl::GenBuffers(1, &mut vbo);

			gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
			gl::BindVertexArray(vao);

			gl::EnableVertexAttribArray(0);
			gl::VertexAttribPointer(
				0, 2, gl::FLOAT, gl::FALSE,
				(2 * size_of::<f32>()) as _,
				0 as _
			);

			let mut v: Vec<f32> = vec![];
			for vertex in vertices
			{
				v.push(vertex.x * World::m2p);
				v.push(vertex.y * World::m2p);
			}

			gl::BufferData(
				gl::ARRAY_BUFFER,
				(v.len() * size_of::<f32>()) as _,
				v.as_ptr() as _,
				gl::DYNAMIC_DRAW
			);

			gl::DrawArrays(gl::TRIANGLE_FAN, 0, (v.len() / 2) as _);

			gl::BindVertexArray(0);
			gl::BindBuffer(gl::ARRAY_BUFFER, 0);
			gl::DeleteVertexArrays(1, &mut vao);
			gl::DeleteBuffers(1, &mut vbo);
		}

		shader.setInt("layer", shader.getInt("layer") + 1);
	}

	fn draw_circle(&mut self, _center: &wrapped2d::b2::Vec2, _radius: f32, _color: &wrapped2d::b2::Color)
	{
		println!("DrawCircle");
	}

	fn draw_segment(&mut self, _p1: &wrapped2d::b2::Vec2, _p2: &wrapped2d::b2::Vec2, _color: &wrapped2d::b2::Color)
	{
		println!("DrawSegment");
	}

	fn draw_solid_circle(&mut self, _center: &wrapped2d::b2::Vec2, _radius: f32, _axis: &wrapped2d::b2::Vec2, _color: &wrapped2d::b2::Color)
	{
		println!("DrawSolidCircle");
	}

	fn draw_transform(&mut self, _xf: &wrapped2d::b2::Transform)
	{
		println!("DrawTransform");
	}
}