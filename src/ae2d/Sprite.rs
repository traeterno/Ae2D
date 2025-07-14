use std::collections::HashMap;

use glam::Vec4Swizzles;

use crate::ae2d::Camera::Drawable;

use super::{Transformable::Transformable2D, Window::Window};

#[derive(Clone, Debug)]
pub struct Animation
{
	repeat: i32,
	repeated: i32,
	frames: Vec<(u8, f32)>,
	currentTime: f32,
	currentFrame: usize
}

impl Animation
{
	pub fn new() -> Self
	{
		Self
		{
			repeat: 0,
			repeated: 0,
			frames: vec![],
			currentFrame: 0,
			currentTime: 0.0
		}
	}

	pub fn parse(base: &json::JsonValue) -> Self
	{
		let mut anim = Animation::new();

		for (var, value) in base.entries()
		{
			if var == "repeat"
			{
				anim.repeat = value.as_i32().unwrap();
			}
			if var == "frames"
			{
				for f in value.members()
				{
					let mut id = 0;
					let mut duration = 0.0;
					for (x, y) in f.entries()
					{
						if x == "id"
						{
							id = y.as_u8().unwrap();
						}
						if x == "duration"
						{
							duration = y.as_f32().unwrap();
						}
					}
					anim.frames.push((id, duration));
				}
			}
		}

		anim
	}

	pub fn update(&mut self)
	{
		if self.repeated >= self.repeat && self.repeat != 0 { return; }

		self.currentTime += crate::ae2d::Window::Window::getDeltaTime();
		if self.currentTime >= self.frames[self.currentFrame].1
		{
			self.currentTime -= self.frames[self.currentFrame].1;
			self.currentFrame += 1;
		}
		if self.currentFrame > self.frames.len() - 1
		{
			if self.repeat == 0 { self.currentFrame = 0; self.currentTime = 0.0; }
			else
			{
				self.repeated += 1;
			}
		}
		self.currentFrame = self.currentFrame.clamp(0, self.frames.len() - 1);
	}

	pub fn getCurrentFrame(&self) -> u8 { self.frames[self.currentFrame].0 }
}

#[derive(Clone)]
pub struct Sprite
{
	animations: HashMap<String, Animation>,
	currentAnimation: String,
	frames: Vec<glam::Vec4>,
	texture: u32,
	vbo: u32,
	vao: u32,
	rectXY: glam::Vec2,
	texSize: glam::Vec2,
	ts: Transformable2D
}

impl Sprite
{
	pub fn default() -> Self
	{
		let mut vao = 0;
		let mut vbo = 0;
		unsafe
		{
			gl::GenVertexArrays(1, &mut vao);
			gl::GenBuffers(1, &mut vbo);

			gl::BindVertexArray(vao);
			gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

			gl::EnableVertexAttribArray(0);
			gl::VertexAttribPointer(
				0, 2, gl::FLOAT,
				gl::FALSE,
				(2 * size_of::<f32>()) as _,
				0 as _
			);
		}
		Self
		{
			animations: HashMap::new(),
			currentAnimation: String::new(),
			frames: vec![],
			texture: 0,
			vbo, vao,
			rectXY: glam::Vec2::ZERO,
			texSize: glam::Vec2::ZERO,
			ts: Transformable2D::new()
		}
	}

	pub fn animated(path: String) -> Self
	{
		let mut spr = Self::default();

		let src = json::parse(
			&std::fs::read_to_string(path).unwrap_or(String::new())
		);
		if src.is_err() { return spr; }
		let src = src.unwrap();

		let mut frame = glam::ivec2(0, 0);
		let mut w = 0;
		let mut h = 0;

		for (section, value) in src.entries()
		{
			if section == "texture"
			{
				spr.texture = Window::getTexture(value.as_str().unwrap().to_string());
				unsafe
				{
					gl::BindTexture(gl::TEXTURE_2D, spr.texture);
					gl::GetTexLevelParameteriv(
						gl::TEXTURE_2D, 0,
						gl::TEXTURE_WIDTH, &mut w
					);
					gl::GetTexLevelParameteriv(
						gl::TEXTURE_2D, 0,
						gl::TEXTURE_HEIGHT, &mut h
					);

					let vertices = [
						0.0, 0.0,
						w as f32, 0.0,
						w as f32, h as f32,
						0.0, h as f32
					];
					
					gl::BindBuffer(gl::ARRAY_BUFFER, spr.vbo);
					gl::BufferData(gl::ARRAY_BUFFER,
						(8 * size_of::<f32>()) as _,
						vertices.as_ptr() as *const _,
						gl::STATIC_DRAW
					);
				}
			}
			if section == "size"
			{
				let mut s = value.members();
				frame = glam::ivec2(
					s.nth(0).unwrap().as_i32().unwrap(),
					s.nth(0).unwrap().as_i32().unwrap()
				);
			}
			if section == "anims"
			{
				for (name, data) in value.entries()
				{
					spr.animations.insert(
						name.to_string(),
						Animation::parse(data)
					);
				}
			}
		}

		spr.texSize = glam::vec2(w as f32, h as f32);
		
		spr.calculateFrames((w, h), frame);

		spr
	}

	pub fn image(path: String) -> Self
	{
		let mut spr = Sprite::default();
		spr.texture = Window::getTexture(path);
		let mut w = 0;
		let mut h = 0;
		unsafe
		{
			gl::BindTexture(gl::TEXTURE_2D, spr.texture);
			gl::GetTexLevelParameteriv(
				gl::TEXTURE_2D, 0,
				gl::TEXTURE_WIDTH, &mut w
			);
			gl::GetTexLevelParameteriv(
				gl::TEXTURE_2D, 0,
				gl::TEXTURE_HEIGHT, &mut h
			);

			let vertices = [
				0.0, 0.0,
				w as f32, 0.0,
				w as f32, h as f32,
				0.0, h as f32
			];

			gl::BindBuffer(gl::ARRAY_BUFFER, spr.vbo);
			gl::BufferData(gl::ARRAY_BUFFER,
				(8 * size_of::<f32>()) as _,
				vertices.as_ptr() as *const _,
				gl::STATIC_DRAW
			);
		}
		spr.texSize = glam::vec2(w as f32, h as f32);
		spr
	}

	pub fn update(&mut self)
	{
		if let Some(anim) = self.animations.get_mut(&self.currentAnimation)
		{
			anim.update();
		}
	}

	fn calculateFrames(&mut self, size: (i32, i32), frame: glam::IVec2)
	{
		self.frames.clear();
		let mut x = 0;
		let mut y = 0;
		while y < size.1
		{
			while x < size.0
			{
				self.frames.push(glam::vec4(
					x as f32,
					y as f32,
					frame.x as f32,
					frame.y as f32
				));
				x += frame.x;
			}
			y += frame.y;
			x = 0;
		}
	}

	pub fn getCurrentFrame(&mut self) -> glam::Vec4
	{
		if self.frames.len() == 0 { return glam::Vec4::ZERO; }
		if self.animations.len() == 0 { return glam::Vec4::ZERO; }
		self.frames[self.animations.get(&self.currentAnimation)
		.unwrap().getCurrentFrame() as usize]
	}

	pub fn setAnimation(&mut self, name: String)
	{
		if name == self.currentAnimation { return; }
		if self.animations.get(&name).is_none() { return; }
		self.currentAnimation = name;
		self.restart();
	}

	fn restart(&mut self)
	{
		if let Some(x) = self.animations.get_mut(&self.currentAnimation)
		{
			x.currentFrame = 0;
			x.currentTime = 0.0;
		}
	}

	pub fn setTextureRect(&mut self, rect: glam::Vec4)
	{
		self.rectXY = rect.xy();
		unsafe
		{
			gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
			gl::BufferData(
				gl::ARRAY_BUFFER,
				(8 * size_of::<f32>()) as _,
				([
					0.0, 0.0,
					rect.z, 0.0,
					rect.z, rect.w,
					0.0, rect.w
				] as [f32; 8]).as_ptr() as *const _,
				gl::STATIC_DRAW
			);
		}
	}

	pub fn getTransformable(&mut self) -> &mut Transformable2D
	{
		&mut self.ts
	}

	pub fn getFrameSize(&self) -> glam::Vec2
	{
		self.texSize
	}
}

impl Drawable for Sprite
{
	fn draw(&mut self)
	{
		self.update();
		let s = Window::getCamera().getImgShader();
		s.activate();
		s.setInt("tex", 0);
		s.setVec2("frameXY",
			if self.animations.len() == 0
			{
				self.rectXY
			}
			else { self.getCurrentFrame().xy() }
		);
		s.setVec2("texSize", self.texSize);
		s.setMat4("model", self.ts.getMatrix());
		s.setVec4("color", glam::Vec4::ONE);
		unsafe
		{
			gl::BindVertexArray(self.vao);
			gl::ActiveTexture(gl::TEXTURE0);
			gl::BindTexture(gl::TEXTURE_2D, self.texture);
			gl::DrawArrays(gl::QUADS, 0, 4);
		}
	}
}

impl Drop for Sprite
{
	fn drop(&mut self)
	{
		unsafe
		{
			gl::DeleteBuffers(1, &mut self.vbo);
			gl::DeleteVertexArrays(1, &mut self.vao);
		}
	}
}