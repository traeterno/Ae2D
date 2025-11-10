use crate::ae2d::Shader::Shader;

use super::{Transformable::Transformable2D, Window::Window};

pub trait Drawable
{
	fn draw(&mut self);
}

pub struct Camera
{
	ts: Transformable2D,
	fbo: u32,
	sbuf: u32,
	tex: u32,
	vao: u32,
	vbo: u32,
	uiProj: glam::Mat4,
	worldProj: glam::Mat4,
	size: glam::Vec2,
	useTS: bool,
	uniVAO: bool,
	activeShader: String
}

impl Camera
{
	pub fn new() -> Self
	{
		Self
		{
			ts: Transformable2D::new(),
			fbo: 0,
			sbuf: 0,
			tex: 0,
			vao: 0,
			vbo: 0,
			uiProj: glam::Mat4::IDENTITY,
			worldProj: glam::Mat4::IDENTITY,
			size: glam::Vec2::ZERO,
			useTS: false,
			uniVAO: false,
			activeShader: String::new()
		}
	}

	pub fn load(&mut self)
	{
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

			gl::GenTextures(1, &mut self.sbuf);
			gl::BindTexture(gl::TEXTURE_2D, self.sbuf);
			gl::TexImage2D(
				gl::TEXTURE_2D, 0, gl::DEPTH24_STENCIL8 as i32,
				w, h, 0,
				gl::DEPTH_STENCIL, gl::UNSIGNED_INT_24_8, 0 as _
			);

			gl::FramebufferTexture2D(
				gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT,
				gl::TEXTURE_2D, self.sbuf, 0
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
				0.0, 0.0,
				1.0, 0.0,
				1.0, 1.0,
				0.0, 1.0
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
		Window::getProfiler().restart();
		unsafe
		{
			gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
			gl::Clear(gl::COLOR_BUFFER_BIT);
			gl::Finish();
		}
		Window::getProfiler().save("clear".to_string());
	}

	pub fn display(&mut self)
	{
		Window::getProfiler().restart();
		unsafe
		{
			gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
			let s = Window::getCamera()
				.activateShader(String::from("camera"));
			s.setInt("tex", 0);
			gl::BindTexture(gl::TEXTURE_2D, self.tex);
			gl::BindVertexArray(self.vao);
			gl::DrawArrays(gl::TRIANGLE_FAN, 0, 4);
			gl::Finish();
		}
		Window::getProfiler().save("render".to_string());
	}

	pub fn toggleTransform(&mut self, enable: bool)
	{
		self.useTS = enable;
		let proj = if enable { self.worldProj } else { self.uiProj };
		let view = if enable {self.ts.getMatrix()} else {glam::Mat4::IDENTITY};
		Window::updateMatrices(proj, view);
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
			self.size = glam::vec2(s.0 as f32, s.1 as f32);
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
				gl::BindTexture(gl::TEXTURE_2D, self.sbuf);
				gl::TexImage2D(
					gl::TEXTURE_2D, 0, gl::DEPTH24_STENCIL8 as i32,
					s.0, s.1, 0,
					gl::DEPTH_STENCIL, gl::UNSIGNED_INT_24_8, 0 as _
				);
			}
		}

	}

	pub fn draw(&mut self, obj: &mut impl Drawable)
	{
		obj.draw();
	}

	pub fn universalVAO(&mut self)
	{
		if self.uniVAO { return; }
		self.uniVAO = true;
		unsafe { gl::BindVertexArray(self.vao); }
	}

	pub fn bindVAO(&mut self, vao: u32)
	{
		self.uniVAO = false;
		unsafe { gl::BindVertexArray(vao); }
	}

	pub fn getTransformable(&mut self) -> &mut Transformable2D
	{
		&mut self.ts
	}

	pub fn getSize(&self) -> glam::Vec2
	{
		self.size
	}

	pub fn getBounds(&mut self) -> glam::Vec4
	{
		let s =
			if self.useTS { self.getSize() }
			else { glam::vec2(Window::getSize().0 as f32, Window::getSize().1 as f32) };
		let m = self.ts.getMatrix();
		let p1 = m * glam::vec4(0.0, 0.0, 0.0, 1.0);
		let p2 = m * glam::vec4(s.x, 0.0, 0.0, 1.0);
		let p3 = m * glam::vec4(s.x, s.y, 0.0, 1.0);
		let p4 = m * glam::vec4(0.0, s.y, 0.0, 1.0);
		
		let min = p1.min(p2).min(p3).min(p4);
		let max = p1.max(p2).max(p3).max(p4);
		glam::vec4(-min.x, -min.y, max.x - min.x, max.y - min.y)
	}

	pub fn activateShader(&mut self, shader: String) -> &'static Shader
	{
		let s = Window::getShader(shader.clone());
		if self.activeShader != shader
		{
			s.activate();
			self.activeShader = shader;
		}
		s
	}

	pub fn isVisible(&mut self, h2: glam::Vec4) -> bool
	{
		let h1 = if self.useTS { self.getBounds() }
		else
		{
			glam::vec4(0.0, 0.0,
				Window::getSize().0 as f32,
				Window::getSize().1 as f32
			)
		};
		let il = h1.x.max(h2.x);
		let it = h1.y.max(h2.y);
		let ir = (h1.x + h1.z).min(h2.x + h2.z);
		let ib = (h1.y + h1.w).min(h2.y + h2.w);

		(il < ir) && (it < ib)
	}
}