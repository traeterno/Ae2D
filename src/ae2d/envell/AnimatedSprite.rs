use crate::ae2d::{Camera::Drawable, FrameAnimation::Animator, Transformable::Transformable2D, Window::Window};

pub struct AnimatedSprite
{
	anim: Animator,
	vao: u32,
	vbo: u32,
	ts: Transformable2D,
	color: glam::Vec4,
}

impl AnimatedSprite
{
	pub fn new() -> Self
	{
		Self
		{
			anim: Animator::new(),
			vao: 0,
			vbo: 0,
			ts: Transformable2D::new(),
			color: glam::Vec4::ONE,
		}
	}

	pub fn loadAnimator(&mut self, path: String) { self.anim.load(path); }
	pub fn setAnimation(&mut self, anim: String) { self.anim.setCurrentAnimation(anim); }
	pub fn getTransform(&mut self) -> &mut Transformable2D { &mut self.ts }
	pub fn getAnimator(&mut self) -> &mut Animator { &mut self.anim }

	fn update(&mut self)
	{
		if self.vao == 0
		{
			unsafe
			{
				gl::GenVertexArrays(1, &mut self.vao);
				gl::GenBuffers(1, &mut self.vbo);

				gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
				gl::BindVertexArray(self.vao);

				gl::EnableVertexAttribArray(0);
				gl::EnableVertexAttribArray(1);

				gl::VertexAttribPointer(
					0, 4, gl::FLOAT, gl::FALSE,
					(8 * size_of::<f32>()) as i32,
					0 as _
				);
				gl::VertexAttribPointer(
					1, 4, gl::FLOAT, gl::FALSE,
					(8 * size_of::<f32>()) as i32,
					(4 * size_of::<f32>()) as _
				);
			}
		}

		if self.anim.update()
		{
			let frame = self.anim.getCurrentFrame();
			let size = self.anim.getFrameSize();
			let vertices = [
				0.0, 0.0,						frame.left(), frame.top(),
				self.color.x, self.color.y, self.color.z, self.color.w,
	
				size.x as f32, 0.0,				frame.right(), frame.top(),
				self.color.x, self.color.y, self.color.z, self.color.w,
				
				size.x as f32, size.y as f32,	frame.right(), frame.bottom(),
				self.color.x, self.color.y, self.color.z, self.color.w,
				
				0.0, size.y as f32,				frame.left(), frame.bottom(),
				self.color.x, self.color.y, self.color.z, self.color.w
			];
			unsafe
			{
				gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
				gl::BufferData(
					gl::ARRAY_BUFFER,
					(32 * size_of::<f32>()) as _,
					vertices.as_ptr() as _,
					gl::DYNAMIC_DRAW
				);
			}
		}
	}

	pub fn getBounds(&mut self) -> sdl2::rect::FRect
	{
		let size = self.anim.getFrameSize();
		let p1 = self.ts.getMatrix() * glam::vec4(0.0, 0.0, 0.0, 1.0);
		let p2 = self.ts.getMatrix() * glam::vec4(size.x as f32, 0.0, 0.0, 1.0);
		let p3 = self.ts.getMatrix() * glam::vec4(size.x as f32, size.y as f32, 0.0, 1.0);
		let p4 = self.ts.getMatrix() * glam::vec4(0.0, size.y as f32, 0.0, 1.0);

		let min = p1.min(p2).min(p3).min(p4);
		let max = p1.max(p2).max(p3).max(p4);

		sdl2::rect::FRect::new(min.x, min.y, max.x - min.x, max.y - min.y)
	}
}

impl Drawable for AnimatedSprite
{
	fn draw(&mut self)
	{
		let shader = Window::getCamera().getImgShader();
		self.update();
		shader.activate();
		shader.setInt("tex", 0);
		shader.setMat4("model", &self.ts.getMatrix().to_cols_array());
		shader.setVec4("color", self.color.to_array());
		unsafe
		{
			gl::BindVertexArray(self.vao);
			gl::ActiveTexture(gl::TEXTURE0);
			self.anim.bindTexture();
			gl::DrawArrays(gl::QUADS, 0, 4);
			gl::BindVertexArray(0);
			gl::BindBuffer(gl::ARRAY_BUFFER, 0);
		}
		shader.setInt("layer", shader.getInt("layer") + 1);
	}
}