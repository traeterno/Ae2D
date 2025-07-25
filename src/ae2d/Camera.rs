use super::{Shader::Shader, Transformable::Transformable2D, Window::Window};

pub trait Drawable
{
	fn draw(&mut self);
}

pub struct Camera
{
	imgShader: Shader,
	txtShader: Shader,
	shapeShader: Shader,
	cameraShader: Shader,
	ts: Transformable2D,
	fbo: u32,
	tex: u32,
	vao: u32,
	vbo: u32,
	uiProj: glam::Mat4,
	worldProj: glam::Mat4
}

impl Camera
{
	pub fn new() -> Self
	{
		Self
		{
			imgShader: Shader::new(),
			txtShader: Shader::new(),
			shapeShader: Shader::new(),
			cameraShader: Shader::new(),
			ts: Transformable2D::new(),
			fbo: 0,
			tex: 0,
			vao: 0,
			vbo: 0,
			uiProj: glam::Mat4::IDENTITY,
			worldProj: glam::Mat4::IDENTITY
		}
	}

	pub fn load(&mut self)
	{
		self.imgShader.load("res/shaders/image.vert", "res/shaders/image.frag");
		self.txtShader.load("res/shaders/text.vert", "res/shaders/text.frag");
		self.shapeShader.load("res/shaders/shape.vert", "res/shaders/shape.frag");
		self.cameraShader.load("res/shaders/camera.vert", "res/shaders/camera.frag");

		let (w, h) = Window::getSize();

		unsafe
		{
			gl::GenFramebuffers(1, &mut self.fbo);
			gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);

			gl::GenTextures(1, &mut self.tex);
			gl::BindTexture(gl::TEXTURE_2D, self.tex);
			gl::TexImage2D(
				gl::TEXTURE_2D, 0, gl::RGB as i32,
				w, h, 0,
				gl::RGB, gl::UNSIGNED_BYTE, 0 as _
			);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

			gl::FramebufferTexture2D(
				gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0,
				gl::TEXTURE_2D, self.tex, 0
			);
			
			gl::GenVertexArrays(1, &mut self.vao);
			gl::GenBuffers(1, &mut self.vbo);

			gl::BindVertexArray(self.vao);
			gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);

			gl::EnableVertexAttribArray(0);

			gl::VertexAttribPointer(
				0, 2, gl::FLOAT,
				gl::FALSE,
				(2 * size_of::<f32>()) as i32,
				0 as _
			);
			
			let vertices: [f32; 8] = [
				-1.0, -1.0,
				1.0, -1.0,
				1.0, 1.0,
				-1.0, 1.0
			];

			gl::BufferData(gl::ARRAY_BUFFER,
				(8 * size_of::<f32>()) as _,
				vertices.as_ptr() as _,
				gl::STATIC_DRAW
			);
		}

		self.setSize(false, (w, h));
		self.toggleTransform(true);
	}

	pub fn clear(&mut self)
	{
		unsafe
		{
			gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
			gl::Clear(gl::COLOR_BUFFER_BIT);
		}
	}

	pub fn display(&mut self)
	{
		unsafe
		{
			gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
			self.cameraShader.activate();
			self.cameraShader.setInt("tex", 0);
			gl::BindTexture(gl::TEXTURE_2D, self.tex);
			gl::BindVertexArray(self.vao);
			gl::DrawArrays(gl::TRIANGLE_FAN, 0, 4);
		}
	}

	pub fn toggleTransform(&mut self, enable: bool)
	{
		let m = if enable { self.worldProj } else { self.uiProj };
		self.imgShader.activate();
		self.imgShader.setMat4("projection", m);
		self.txtShader.activate();
		self.txtShader.setMat4("projection", m);
		self.shapeShader.activate();
		self.shapeShader.setMat4("projection", m);
		let m = if enable {self.ts.getMatrix()} else {glam::Mat4::IDENTITY};
		self.imgShader.activate();
		self.imgShader.setMat4("view", m);
		self.txtShader.activate();
		self.txtShader.setMat4("view", m);
		self.shapeShader.activate();
		self.shapeShader.setMat4("view", m);
	}

	pub fn setSize(&mut self, mode: bool, s: (i32, i32))
	{
		let m = glam::Mat4::orthographic_rh_gl(
			0.0, s.0 as f32,
			s.1 as f32, 0.0,
			-1.0, 1.0
		);

		if mode
		{
			self.worldProj = m;
		}
		else
		{
			self.uiProj = m;
			unsafe
			{
				gl::BindTexture(gl::TEXTURE_2D, self.tex);
				gl::TexImage2D(
					gl::TEXTURE_2D, 0, gl::RGB as i32,
					s.0, s.1, 0, gl::RGB,
					gl::UNSIGNED_BYTE, 0 as _
				);
			}
		}

	}

	pub fn draw(&mut self, obj: &mut impl Drawable)
	{
		obj.draw();
	}

	pub fn getTransformable(&mut self) -> &mut Transformable2D
	{
		&mut self.ts
	}
	
	pub fn getImgShader(&mut self) -> &mut Shader { &mut self.imgShader }
	pub fn getTxtShader(&mut self) -> &mut Shader { &mut self.txtShader }
	// pub fn getShapeShader(&mut self) -> &mut Shader { &mut self.shapeShader }
}