use glam::Vec4Swizzles;
use stb_image::image::Image;

use crate::ae2d::{Camera::Drawable, Transformable::Transformable2D, Window::Window};

pub struct Sprite
{
	texture: u32,
	texSize: glam::Vec2,
	vao: u32,
	vbo: u32,
	ts: Transformable2D,
	color: glam::Vec4,
	rect: glam::Vec4,
	reload: bool
}

impl Sprite
{
	pub fn new() -> Self
	{
		Self
		{
			texture: 0,
			texSize: glam::Vec2::ZERO,
			vao: 0,
			vbo: 0,
			ts: Transformable2D::new(),
			color: glam::Vec4::ONE,
			rect: glam::Vec4::ZERO,
			reload: true
		}
	}

	pub fn setTexture(&mut self, img: Image<u8>)
	{
		if self.texture != 0 { unsafe { gl::DeleteTextures(1, &mut self.texture); } }
		
		unsafe
		{
			gl::GenTextures(1, &mut self.texture);
			gl::BindTexture(gl::TEXTURE_2D, self.texture);

			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

			gl::TexImage2D(
				gl::TEXTURE_2D,
				0,
				gl::RGBA as i32,
				img.width as i32,
				img.height as i32,
				0,
				gl::RGBA,
				gl::UNSIGNED_BYTE,
				img.data.as_ptr() as *const _
			);
			
			gl::BindTexture(gl::TEXTURE_2D, 0);
		}

		self.texSize = glam::vec2(img.width as f32, img.height as f32);

		self.rect = glam::vec4(0.0, 0.0, img.width as f32, img.height as f32);
		self.reload = true;
	}

	fn update(&mut self)
	{
		self.reload = false;

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

		let l = self.rect.x;
		let t = self.rect.y;
		let r = self.rect.x + self.rect.z;
		let b = self.rect.y + self.rect.w;
		let w = self.rect.z.abs();
		let h = self.rect.w.abs();

		let vertices = [
			0.0, 0.0, l / self.texSize.x, t / self.texSize.y,
			self.color.x, self.color.y, self.color.z, self.color.w,

			w, 0.0, r / self.texSize.x, t / self.texSize.y,
			self.color.x, self.color.y, self.color.z, self.color.w,

			w, h, r / self.texSize.x, b / self.texSize.y,
			self.color.x, self.color.y, self.color.z, self.color.w,

			0.0, h, l / self.texSize.x, b / self.texSize.y,
			self.color.x, self.color.y, self.color.z, self.color.w,
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

	pub fn setTextureRect(&mut self, rect: glam::Vec4)
	{
		self.rect = rect;
		self.reload = true;
	}

	pub fn getBounds(&mut self) -> sdl2::rect::FRect
	{
		let size = self.rect.zw();
		let p1 = self.ts.getMatrix() * glam::vec4(0.0, 0.0, 0.0, 1.0);
		let p2 = self.ts.getMatrix() * glam::vec4(size.x as f32, 0.0, 0.0, 1.0);
		let p3 = self.ts.getMatrix() * glam::vec4(size.x as f32, size.y as f32, 0.0, 1.0);
		let p4 = self.ts.getMatrix() * glam::vec4(0.0, size.y as f32, 0.0, 1.0);

		let min = p1.min(p2).min(p3).min(p4);
		let max = p1.max(p2).max(p3).max(p4);

		sdl2::rect::FRect::new(min.x, min.y, max.x - min.x, max.y - min.y)
	}

	pub fn getTransform(&mut self) -> &mut Transformable2D { &mut self.ts }
}

impl Drawable for Sprite
{
	fn draw(&mut self)
	{
		if self.reload { self.update(); }

		let shader = Window::getCamera().getImgShader();
		shader.activate();
		shader.setInt("tex", 0);
		shader.setMat4("model", &self.ts.getMatrix().to_cols_array());
		shader.setVec4("color", self.color.to_array());
		unsafe
		{
			gl::BindVertexArray(self.vao);
			gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
			gl::ActiveTexture(gl::TEXTURE0);
			gl::BindTexture(gl::TEXTURE_2D, self.texture);
			gl::DrawArrays(gl::QUADS, 0, 4);
			gl::BindVertexArray(0);
			gl::BindBuffer(gl::ARRAY_BUFFER, 0);
		}
		shader.setInt("layer", shader.getInt("layer") + 1);
	}
}