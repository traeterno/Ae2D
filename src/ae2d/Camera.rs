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
	size: (i32, i32),
	fbo: u32,
	tex: u32,
	zbuf: u32,
	vao: u32,
	vbo: u32,
	layer: i32
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
			size: (0, 0),
			fbo: 0,
			tex: 0,
			zbuf: 0,
			vao: 0,
			vbo: 0,
			layer: 0
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

			gl::GenTextures(1, &mut self.zbuf);
			gl::BindTexture(gl::TEXTURE_2D, self.zbuf);

			gl::TexImage2D(
				gl::TEXTURE_2D, 0, gl::DEPTH_COMPONENT as i32,
				w, h, 0,
				gl::DEPTH_COMPONENT, gl::UNSIGNED_INT, 0 as _
			);

			gl::FramebufferTexture2D(
				gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT,
				gl::TEXTURE_2D, self.zbuf, 0
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

		self.setSize((w, h));
		self.toggleTransform(true);
	}

	pub fn clear(&mut self)
	{
		self.layer = -99;
		self.imgShader.activate();
		self.imgShader.setInt("layer", self.layer);
		self.txtShader.activate();
		self.txtShader.setInt("layer", self.layer);
		self.shapeShader.activate();
		self.shapeShader.setInt("layer", self.layer);
		unsafe
		{
			gl::Enable(gl::DEPTH_TEST);
			gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
			gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
			
		}
	}

	pub fn display(&mut self)
	{
		unsafe
		{
			gl::Disable(gl::DEPTH_TEST);
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
		let m = if enable {self.ts.getMatrix()} else {glam::Mat4::IDENTITY};
		self.imgShader.activate();
		self.imgShader.setMat4("view", m);
		self.txtShader.activate();
		self.txtShader.setMat4("view", m);
		self.shapeShader.activate();
		self.shapeShader.setMat4("view", m);
	}

	pub fn setSize(&mut self, s: (i32, i32))
	{
		self.size = s;
		let m = glam::Mat4::orthographic_rh_gl(
			0.0, s.0 as f32,
			s.1 as f32, 0.0,
			-100.0, 100.0
		);
		self.imgShader.activate();
		self.imgShader.setMat4("projection", m);
		self.txtShader.activate();
		self.txtShader.setMat4("projection", m);
		self.shapeShader.activate();
		self.shapeShader.setMat4("projection", m);

		unsafe
		{
			gl::BindTexture(gl::TEXTURE_2D, self.tex);
			gl::TexImage2D(
				gl::TEXTURE_2D, 0, gl::RGB as i32,
				s.0, s.1, 0, gl::RGB,
				gl::UNSIGNED_BYTE, 0 as _
			);

			gl::BindTexture(gl::TEXTURE_2D, self.zbuf);
			gl::TexImage2D(
				gl::TEXTURE_2D, 0, gl::DEPTH_COMPONENT as i32,
				s.0, s.1, 0, gl::DEPTH_COMPONENT,
				gl::UNSIGNED_BYTE, 0 as _
			);
		}
	}

	pub fn draw(&mut self, obj: &mut impl Drawable)
	{
		obj.draw();
		self.layer += 1;
		self.imgShader.activate();
		self.imgShader.setInt("layer", self.layer);
		self.txtShader.activate();
		self.txtShader.setInt("layer", self.layer);
		self.shapeShader.activate();
		self.shapeShader.setInt("layer", self.layer);
	}
	pub fn getImgShader(&mut self) -> &mut Shader { &mut self.imgShader }
	pub fn getTxtShader(&mut self) -> &mut Shader { &mut self.txtShader }
	// pub fn getShapeShader(&mut self) -> &mut Shader { &mut self.shapeShader }
}