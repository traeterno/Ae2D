use std::collections::HashMap;

use super::{Camera::Drawable, Transformable::Transformable2D, Window::Window};

#[derive(Clone)]
struct Glyph
{
	rect: glam::Vec4,
	offset: glam::Vec2,
	advance: u8
}

struct Font
{
	texture: u32,
	glyphs: HashMap<u16, Glyph>,
	height: f32,
	bitmapSize: glam::Vec2
}

impl Font
{
	pub fn default() -> Self
	{
		Self
		{
			texture: 0,
			glyphs: HashMap::new(),
			height: 0.0,
			bitmapSize: glam::Vec2::ZERO
		}
	}

	pub fn load(path: String) -> Self
	{
		let mut font = Self::default();

		let f = json::parse(
			&std::fs::read_to_string(path).unwrap()
		);
		if f.is_err() { return font; }
		let f = f.unwrap();

		let mut w = 0;
		let mut h = 0;

		for section in f.entries()
		{
			if section.0 == "lineHeight" { font.height = section.1.as_f32().unwrap(); }
			if section.0 == "texture"
			{
				font.texture = Window::getTexture(
					section.1.as_str().unwrap().to_string()
				);
				unsafe
				{
					gl::BindTexture(gl::TEXTURE_2D, font.texture);
					gl::GetTexLevelParameteriv(
						gl::TEXTURE_2D, 0,
						gl::TEXTURE_WIDTH, &mut w
					);
					gl::GetTexLevelParameteriv(
						gl::TEXTURE_2D, 0,
						gl::TEXTURE_HEIGHT, &mut h
					);
				}
			}
			if section.0 == "glyphs"
			{
				for glyph in section.1.members()
				{
					let mut id = 0;
					let mut g = Glyph
					{
						rect: glam::Vec4::ZERO,
						offset: glam::Vec2::ZERO,
						advance: 0
					};
					for x in glyph.entries()
					{
						if x.0 == "id" { id = x.1.as_u16().unwrap(); }
						if x.0 == "x" { g.rect.x = x.1.as_f32().unwrap(); }
						if x.0 == "y" { g.rect.y = x.1.as_f32().unwrap(); }
						if x.0 == "w" { g.rect.z = x.1.as_f32().unwrap(); }
						if x.0 == "h" { g.rect.w = x.1.as_f32().unwrap(); }
						if x.0 == "ox" { g.offset.x = x.1.as_f32().unwrap(); }
						if x.0 == "oy" { g.offset.y = x.1.as_f32().unwrap(); }
						if x.0 == "advance" { g.advance = x.1.as_u8().unwrap(); }
					}
					font.glyphs.insert(id, g);
				}
			}
		}

		font.bitmapSize = glam::vec2(w as f32, h as f32);
		font
	}

	pub fn getGlyph(&self, c: char) -> Option<&Glyph>
	{
		let g = self.glyphs.get(&(c as u16));
		match g
		{
			None => { println!("Символ не найден: {c}({})", c as u16); g },
			Some(_) => g
		}
	}
}

pub struct Text
{
	font: Font,
	vbo: u32,
	vao: u32,
	text: String,
	update: bool,
	ts: Transformable2D,
	vertices: i32,
	size: f32,
	dimensions: glam::Vec2
}

impl Text
{
	pub fn new() -> Self
	{
		let mut vao = 0;
		let mut vbo = 0;
		unsafe
		{
			gl::GenVertexArrays(1, &mut vao);
			gl::GenBuffers(1, &mut vbo);
		}
		Self
		{
			font: Font::default(),
			vao, vbo,
			text: String::new(),
			update: false,
			ts: Transformable2D::new(),
			vertices: 0,
			size: 0.0,
			dimensions: glam::Vec2::ZERO
		}
	}

	pub fn setFont(&mut self, path: String)
	{
		self.font = Font::load(path);
		self.update = true;
	}

	pub fn setString(&mut self, txt: String)
	{
		self.text = txt;
		self.update = true;
	}

	fn reload(&mut self)
	{
		self.update = false;
		let b = self.font.bitmapSize;

		let mut line = Vec::<f32>::new();

		let mut x = 0.0;
		let mut y = 0.0;

		for c in self.text.chars()
		{
			if c == '\n'
			{
				y += self.font.height;
				x = 0.0;
				continue;
			}
			let g = self.font.getGlyph(c);
			if g.is_none() { continue; }
			let g = g.unwrap();
			
			line.append(&mut vec![
				x + g.offset.x, y + g.offset.y,
				g.rect.x / b.x, g.rect.y / b.y,

				x + g.offset.x + g.rect.z, y + g.offset.y,
				(g.rect.x + g.rect.z) / b.x, g.rect.y / b.y,

				x + g.offset.x + g.rect.z, y + g.offset.y + g.rect.w,
				(g.rect.x + g.rect.z) / b.x, (g.rect.y + g.rect.w) / b.y,

				x + g.offset.x, y + g.offset.y + g.rect.w,
				g.rect.x / b.x, (g.rect.y + g.rect.w) / b.y
			]);
			
			x += g.advance as f32;
		}
		
		self.vertices = line.len() as i32 / 4;
		
		let scale = self.size / self.font.height;

		self.dimensions = glam::Vec2::ZERO;
		
		for i in 0..self.vertices as usize
		{
			let x = i * 4;
			line[x] *= scale;
			line[x + 1] *= scale;

			self.dimensions.x = self.dimensions.x.max(line[x]);
			self.dimensions.y = self.dimensions.y.max(line[x + 1]);
		}
		
		unsafe
		{
			gl::BindVertexArray(self.vao);
			gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
			gl::BufferData(gl::ARRAY_BUFFER,
				(line.len() * size_of::<f32>()) as _,
				line.as_ptr() as *const _,
				gl::DYNAMIC_DRAW
			);
			gl::EnableVertexAttribArray(0);
			gl::VertexAttribPointer(
				0, 4, gl::FLOAT,
				gl::FALSE,
				(4 * size_of::<f32>()) as _,
				0 as _
			);
		}
	}

	pub fn getTransformable(&mut self) -> &mut Transformable2D
	{
		&mut self.ts
	}

	pub fn setSize(&mut self, size: f32)
	{
		self.size = size;
		self.update = true;
	}

	pub fn getDimensions(&mut self) -> glam::Vec2
	{
		if self.update { self.reload(); }
		self.dimensions
	}

	pub fn getBounds(&mut self) -> glam::Vec4
	{
		if self.update { self.reload(); }

		let p1 = self.ts.getMatrix() * glam::vec4(0.0, 0.0, 0.0, 1.0);
		let p2 = self.ts.getMatrix() * glam::vec4(self.dimensions.x, 0.0, 0.0, 1.0);
		let p3 = self.ts.getMatrix() * glam::vec4(self.dimensions.x, self.dimensions.y, 0.0, 1.0);
		let p4 = self.ts.getMatrix() * glam::vec4(0.0, self.dimensions.y, 0.0, 1.0);

		let min = p1.min(p2).min(p3).min(p4);
		let max = p1.max(p2).max(p3).max(p4);

		glam::vec4(min.x, min.y, max.x - min.x, max.y - min.y)
	}
}

impl Drawable for Text
{
	fn draw(&mut self)
	{
		if self.update { self.reload(); }
		if self.vertices == 0 { return; }
		let s = Window::getShader(String::from("text"));
		s.activate();
		s.setInt("tex", 0);
		s.setMat4("model", self.ts.getMatrix());
		unsafe
		{
			gl::BindVertexArray(self.vao);
			gl::ActiveTexture(gl::TEXTURE0);
			gl::BindTexture(gl::TEXTURE_2D, self.font.texture);
			gl::DrawArrays(gl::QUADS,
				0, self.vertices
			);
		}
	}
}

impl Drop for Text
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