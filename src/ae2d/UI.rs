use mlua::{Error, Function, Integer, Lua, Number, StdLib};

use crate::ae2d::{Assets, Programmable::{Programmable, Variable}, Window::Window};

use super::{envell::World::World, Camera::Drawable, FrameAnimation::Animator, Transformable::Transformable2D};
pub struct Image
{
	animation: Animator,
	vao: u32,
	vbo: u32,
	vertices: [f32; 32],
	ts: Transformable2D,
	color: glam::Vec4
}

impl Image
{
	pub fn new() -> Self
	{
		Self
		{
			animation: Animator::new(),
			vao: 0,
			vbo: 0,
			vertices: [0.0; 32],
			ts: Transformable2D::new(),
			color: glam::Vec4::ONE
		}
	}

	pub fn parse(node: &spex::xml::Element) -> Self
	{
		let mut img = Image::new();
		if node.name().local_part() != "image" { return img; }
		
		unsafe
		{
			gl::GenVertexArrays(1, &mut img.vao);
			gl::GenBuffers(1, &mut img.vbo);

			gl::BindBuffer(gl::ARRAY_BUFFER, img.vbo);
			gl::BindVertexArray(img.vao);
			gl::EnableVertexAttribArray(0);
			gl::EnableVertexAttribArray(1);
			gl::VertexAttribPointer(
				0, 4, gl::FLOAT,
				gl::FALSE,
				(8 * size_of::<f32>()) as i32,
				std::ptr::null()
			);
			gl::VertexAttribPointer(
				1, 4, gl::FLOAT,
				gl::FALSE,
				(8 * size_of::<f32>()) as i32,
				(4 * size_of::<f32>()) as *const _
			);
		}

		img.animation.load(node.att_opt("anim")
			.unwrap_or_else(|| { println!("No animation provided"); "" })
			.to_string()
		);

		img
	}

	pub fn getBounds(&mut self) -> sdl2::rect::FRect
	{
		let size = self.animation.getFrameSize();
		let p1 = self.ts.getMatrix() * glam::vec4(0.0, 0.0, 0.0, 1.0);
		let p2 = self.ts.getMatrix() * glam::vec4(size.x as f32, 0.0, 0.0, 1.0);
		let p3 = self.ts.getMatrix() * glam::vec4(size.x as f32, size.y as f32, 0.0, 1.0);
		let p4 = self.ts.getMatrix() * glam::vec4(0.0, size.y as f32, 0.0, 1.0);

		let min = p1.min(p2).min(p3).min(p4);
		let max = p1.max(p2).max(p3).max(p4);

		sdl2::rect::FRect::new(min.x, min.y, max.x - min.x, max.y - min.y)
	}

	fn setPositionFN(_: &Lua, options: (f32, f32)) -> Result<(), Error>
	{
		let obj = unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() };
		obj.image.ts.setPosition(glam::vec2(options.0, options.1));
		Ok(())
	}

	fn translateFN(_: &Lua, options: (f32, f32)) -> Result<(), Error>
	{
		let obj = unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() };
		obj.image.ts.translate(glam::vec2(options.0, options.1));
		Ok(())
	}

	fn getPositionFN(_: &Lua, _: ()) -> Result<(Number, Number), Error>
	{
		let obj = unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() };
		Ok((
			obj.image.ts.getPosition().x as f64,
			obj.image.ts.getPosition().y as f64
		))
	}

	fn setRotationFN(_: &Lua, angle: f32) -> Result<(), Error>
	{
		let obj = unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() };
		obj.image.ts.setRotation(angle);
		Ok(())
	}

	fn rotateFN(_: &Lua, angle: f32) -> Result<(), Error>
	{
		let obj = unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() };
		obj.image.ts.rotate(angle);
		Ok(())
	}

	fn getRotationFN(_: &Lua, _: ()) -> Result<Number, Error>
	{
		let obj = unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() };
		Ok(obj.image.ts.getRotation() as f64)
	}

	fn setScaleFN(_: &Lua, s: (f32, f32)) -> Result<(), Error>
	{
		let obj = unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() };
		obj.image.ts.setScale(glam::vec2(s.0, s.1));
		Ok(())
	}

	fn scaleFN(_: &Lua, s: (f32, f32)) -> Result<(), Error>
	{
		let obj = unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() };
		obj.image.ts.scale(glam::vec2(s.0, s.1));
		Ok(())
	}

	fn getScaleFN(_: &Lua, _: ()) -> Result<(Number, Number), Error>
	{
		let obj = unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() };
		Ok((
			obj.image.ts.getScale().x as f64,
			obj.image.ts.getScale().y as f64
		))
	}

	fn setOriginFN(_: &Lua, o: (f32, f32)) -> Result<(), Error>
	{
		let obj = unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() };
		obj.image.ts.setOrigin(glam::vec2(o.0, o.1));
		Ok(())
	}

	fn getOriginFN(_: &Lua, _: ()) -> Result<(Number, Number), Error>
	{
		let obj = unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() };
		Ok((
			obj.image.ts.getOrigin().x as f64,
			obj.image.ts.getOrigin().y as f64
		))
	}

	fn boundsFN(_: &Lua, _: ()) -> Result<(Number, Number, Number, Number), Error>
	{
		let bounds = unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() }.image.getBounds();
		Ok((
			bounds.left() as f64,
			bounds.top() as f64,
			bounds.width() as f64,
			bounds.height() as f64
		))
	}
	
	fn sizeFN(_: &Lua, _: ()) -> Result<(Number, Number), Error>
	{
		let size = unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() }.image.animation.getFrameSize();
		Ok((size.x as f64, size.y as f64))
	}

	fn setAnimationFN(_: &Lua, name: String) -> Result<(), Error>
	{
		let obj = &mut unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() }.image;
		obj.animation.setCurrentAnimation(name);
		Ok(())
	}

	fn setColorFN(_: &Lua, clr: (Integer, Integer, Integer, Integer)) -> Result<(), Error>
	{
		let obj = &mut unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() }.image;
		obj.color = glam::vec4(
			clr.0 as f32 / 255.0,
			clr.1 as f32 / 255.0,
			clr.2 as f32 / 255.0,
			clr.3 as f32 / 255.0
		);
		Ok(())
	}

	pub fn initLua(script: &Lua)
	{
		let table = script.create_table().unwrap();

		table.set("setPosition", script.create_function(Image::setPositionFN).unwrap());
		table.set("translate", script.create_function(Image::translateFN).unwrap());
		table.set("getPosition", script.create_function(Image::getPositionFN).unwrap());

		table.set("setRotation", script.create_function(Image::setRotationFN).unwrap());
		table.set("rotate", script.create_function(Image::rotateFN).unwrap());
		table.set("getRotation", script.create_function(Image::getRotationFN).unwrap());

		table.set("setScale", script.create_function(Image::setScaleFN).unwrap());
		table.set("scale", script.create_function(Image::scaleFN).unwrap());
		table.set("getScale", script.create_function(Image::getScaleFN).unwrap());

		table.set("setOrigin", script.create_function(Image::setOriginFN).unwrap());
		table.set("getOrigin", script.create_function(Image::getOriginFN).unwrap());

		table.set("bounds", script.create_function(Image::boundsFN).unwrap());
		table.set("size", script.create_function(Image::sizeFN).unwrap());

		table.set("setAnimation", script.create_function(Image::setAnimationFN).unwrap());
		table.set("setColor", script.create_function(Image::setColorFN).unwrap());
		
		script.globals().set("image", table);
	}
}

impl Drop for Image
{
	fn drop(&mut self)
	{
		if self.vao == 0 && self.vbo == 0 { return; }
		unsafe
		{
			gl::DeleteVertexArrays(1, &mut self.vao);
			gl::DeleteBuffers(1, &mut self.vbo);
		}
	}
}

impl Drawable for Image
{
	fn draw(&mut self)
	{
		let upd = self.animation.update();
		if upd
		{
			let frame = self.animation.getCurrentFrame();
			let size = self.animation.getFrameSize();
			self.vertices = [
				0.0, 0.0,						frame.left(), frame.top(),
				self.color.x, self.color.y, self.color.z, self.color.w,
	
				size.x as f32, 0.0,				frame.right(), frame.top(),
				self.color.x, self.color.y, self.color.z, self.color.w,
				
				size.x as f32, size.y as f32,	frame.right(), frame.bottom(),
				self.color.x, self.color.y, self.color.z, self.color.w,
				
				0.0, size.y as f32,				frame.left(), frame.bottom(),
				self.color.x, self.color.y, self.color.z, self.color.w
			];
		}
		let shader = Window::getCamera().getImgShader();
		shader.activate();
		shader.setInt("tex", 0);
		shader.setMat4("model", &self.ts.getMatrix().to_cols_array());
		shader.setVec4("color", self.color.to_array());
		unsafe
		{
			gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
			if upd
			{
				gl::BufferData(
					gl::ARRAY_BUFFER,
					(self.vertices.len() * size_of::<f32>()) as isize,
					self.vertices.as_ptr() as *const _,
					gl::DYNAMIC_DRAW
				);
			}

			gl::BindVertexArray(self.vao);
			gl::ActiveTexture(gl::TEXTURE0);
			self.animation.bindTexture();
			gl::DrawArrays(gl::QUADS, 0, 4);
		}
		shader.setInt("layer", shader.getInt("layer") + 1);
	}
}

#[derive(Clone, Debug)]
pub struct Glyph
{
	pub rect: sdl2::rect::FRect,
	pub offset: sdl2::rect::Point,
	pub advance: u8,
}

#[derive(Debug)]
pub struct Font
{
	page: u32,
	glyphs: std::collections::HashMap<u16, Glyph>,
	height: u8,
	name: String,
	base: u8,
	pub bitmapSize: glam::IVec2
}

impl Font
{
	pub fn new() -> Self
	{
		Self
		{
			page: 0,
			glyphs: std::collections::HashMap::new(),
			height: 0,
			name: "".to_string(),
			base: 0,
			bitmapSize: glam::ivec2(0, 0)
		}
	}

	pub fn load(path: String) -> Self
	{
		let mut font = Font::new();
		let src = crate::ae2d::Assets::readXML(path.clone());
		if src.is_none() { return font; }
		
		for node in src.unwrap().elements()
		{
			let name = node.name().local_part();
			if name == "info"
			{
				font.name =
					node.att_req("face")
					.unwrap_or("")
					.to_string();
			}
			if name == "common"
			{
				font.height =
					node.att_req("lineHeight")
					.unwrap_or("0")
					.parse::<u8>()
					.unwrap();
			}
			if name == "pages"
			{
				let mut p = path.clone();
				while p.chars().last().unwrap() != '/' { p.pop(); }
				font.page = crate::ae2d::Assets::getTexture(
					p + node.elements().nth(0).unwrap()
					.att_req("file")
					.unwrap_or("")
				);
				unsafe
				{
					gl::BindTexture(gl::TEXTURE_2D, font.page);
					gl::GetTexLevelParameteriv(gl::TEXTURE_2D, 0, gl::TEXTURE_WIDTH, &mut font.bitmapSize.x);
					gl::GetTexLevelParameteriv(gl::TEXTURE_2D, 0, gl::TEXTURE_HEIGHT, &mut font.bitmapSize.y);
					gl::BindTexture(gl::TEXTURE_2D, 0);
				}
			}
			if name == "chars"
			{
				for ch in node.elements()
				{
					font.glyphs.insert(
						ch.att_req("id")
							.unwrap_or("0")
							.parse::<u16>()
							.unwrap(),
						Glyph
						{
							rect: sdl2::rect::FRect::new(
								ch.att_req("x")
									.unwrap_or("0")
									.parse::<f32>()
									.unwrap(),
								ch.att_req("y")
									.unwrap_or("0")
									.parse::<f32>()
									.unwrap(),
								ch.att_req("width")
									.unwrap_or("0")
									.parse::<f32>()
									.unwrap(),
								ch.att_req("height")
									.unwrap_or("0")
									.parse::<f32>()
									.unwrap()
							),
							offset: sdl2::rect::Point::new(
								ch.att_req("xoffset")
									.unwrap_or("0")
									.parse::<i32>()
									.unwrap(),
								ch.att_req("yoffset")
									.unwrap_or("0")
									.parse::<i32>()
									.unwrap()
							),
							advance: ch.att_req("xadvance")
								.unwrap_or("0")
								.parse::<u8>()
								.unwrap()
						}
					);
				}
			}
		}

		font
	}

	pub fn getGlyph(&mut self, c: char) -> Glyph
	{
		let empty = Glyph
		{
			advance: 0,
			offset: sdl2::rect::Point::new(0, 0),
			rect: sdl2::rect::FRect::new(0.0, 0.0, 0.0, 0.0)
		};

		self.glyphs.get(&(c as u16)).unwrap_or_else(||
			{
				println!("Glyph not found: '{c}' as {}", c as u32);
				&empty
			}
		).clone()
	}

	pub fn bindTexture(&mut self)
	{
		unsafe
		{
			gl::BindTexture(gl::TEXTURE_2D, self.page);
		}
	}

	pub fn unbindTexture()
	{
		unsafe
		{
			gl::BindTexture(gl::TEXTURE_2D, 0);
		}
	}
}

#[derive(Debug)]
struct StyledText
{
	pub text: String,
	pub bold: bool,
	pub italic: bool,
	pub underlined: bool,
	pub strikethrough: bool,
	pub newline: bool,
	pub color: sdl2::pixels::Color,
}

#[derive(Debug)]
pub struct Text
{
	font: Font,
	text: Vec<StyledText>,
	vertices: i32,
	reload: bool,
	vbo: u32,
	vao: u32,
	fontSize: u8,
	dimensions: glam::Vec2,
	ts: Transformable2D
}

impl Drop for Text
{
	fn drop(&mut self)
	{
		if self.vao == 0 && self.vbo == 0 { return; }
		unsafe
		{
			gl::DeleteVertexArrays(1, &mut self.vao);
			gl::DeleteBuffers(1, &mut self.vbo);
		}
	}
}

impl Text
{
	pub fn new() -> Self
	{
		Self
		{
			font: Font::new(),
			text: vec![],
			vbo: 0,
			vao: 0,
			vertices: 0,
			reload: true,
			fontSize: 48,
			dimensions: glam::Vec2::ZERO,
			ts: Transformable2D::new()
		}
	}

	pub fn loadFont(&mut self, path: String)
	{
		if self.vao == 0 && self.vbo == 0
		{
			unsafe
			{
				gl::GenVertexArrays(1, &mut self.vao);
				gl::GenBuffers(1, &mut self.vbo);
			}
		}
		self.font = Font::load(path);
	}

	pub fn setString(&mut self, str: String)
	{
		let mut part = StyledText
		{
			text: String::new(),
			bold: false,
			italic: false,
			underlined: false,
			strikethrough: false,
			newline: false,
			color: sdl2::pixels::Color::WHITE,
		};

		self.text.clear();

		let chars: Vec<char> = str.as_str().chars().collect();
		let mut index = 0;
		while index < chars.len()
		{
			let c = *chars.get(index).unwrap();

			if c == '^' && *chars.get(index + 1).unwrap_or(&' ') == '('
			{
				if !part.text.is_empty()
				{
					self.text.push(part);
					part = StyledText
					{
						text: String::new(),
						bold: false,
						italic: false,
						underlined: false,
						strikethrough: false,
						newline: false,
						color: sdl2::pixels::Color::WHITE,
					};
				}
				let mut raw = String::new();

				index += 2;
				while *chars.get(index).unwrap_or(&')') != ')'
				{
					raw.push(*chars.get(index).unwrap_or(&' '));
					index += 1;
				}

				let style: Vec<&str> = raw.split(" ").collect();
				for el in style
				{
					if el == "*" { part.bold = true; }
					if el == "/" { part.italic = true; }
					if el == "_" { part.underlined = true; }
					if el == "-" { part.strikethrough = true; }
					if el == "<" { part.newline = true; }
					if el.contains("clr")
					{
						part.color = crate::ae2d::Window::Window::getColor(el.split("=").nth(1).unwrap().to_string());
					}
				}
			}
			else { part.text.push(c); }
			
			index += 1;
		}
		self.text.push(part);

		self.reload = true;
	}

	pub fn update(&mut self)
	{
		self.dimensions = glam::Vec2::ZERO;
		let mut vertices: Vec<f32> = vec![];

		let mut pos = glam::Vec2::ZERO;

		let scale = self.fontSize as f32 / self.font.height as f32;
		let italic = self.font.height as f32 * 10.0_f32.to_radians().sin();

		for part in self.text.iter()
		{
			let clr = glam::vec4(
				part.color.r as f32 / 255.0,
				part.color.g as f32 / 255.0,
				part.color.b as f32 / 255.0,
				part.color.a as f32 / 255.0
			);
			for ch in part.text.chars()
			{
				let glyph = self.font.getGlyph(ch);
				vertices.append(&mut vec![
					pos.x + (glyph.offset.x as f32 + if part.italic { italic } else { 0.0 }) * scale,
					pos.y + (glyph.offset.y as f32 - self.font.base as f32) * scale,
					glyph.rect.left() / self.font.bitmapSize.x as f32,
					glyph.rect.top() / self.font.bitmapSize.y as f32,
					clr.x, clr.y, clr.z, clr.w,
	
					pos.x + (glyph.offset.x as f32 + glyph.rect.width() + if part.italic { italic } else { 0.0 }) * scale,
					pos.y + (glyph.offset.y as f32 - self.font.base as f32) * scale,
					glyph.rect.right() / self.font.bitmapSize.x as f32,
					glyph.rect.top() / self.font.bitmapSize.y as f32,
					clr.x, clr.y, clr.z, clr.w,
	
					pos.x + (glyph.offset.x as f32 + glyph.rect.width()) * scale,
					pos.y + (glyph.offset.y as f32 - self.font.base as f32 * scale + glyph.rect.height()) * scale,
					glyph.rect.right() / self.font.bitmapSize.x as f32,
					glyph.rect.bottom() / self.font.bitmapSize.y as f32,
					clr.x, clr.y, clr.z, clr.w,
	
					pos.x + (glyph.offset.x as f32) * scale,
					pos.y + (glyph.offset.y as f32 - self.font.base as f32 * scale + glyph.rect.height()) * scale,
					glyph.rect.left() / self.font.bitmapSize.x as f32,
					glyph.rect.bottom() / self.font.bitmapSize.y as f32,
					clr.x, clr.y, clr.z, clr.w
				]);

				self.dimensions.x = self.dimensions.x.max(
					pos.x + (glyph.offset.x as f32 + glyph.rect.width() + if part.italic { italic } else { 0.0 }) * scale
				);
				self.dimensions.y = self.dimensions.y.max(
					pos.y + (glyph.offset.y as f32 - self.font.base as f32 * scale + glyph.rect.height()) * scale
				);

				pos.x += glyph.advance as f32 * scale;

			}
			if part.newline
			{
				pos.x = 0.0;
				pos.y += self.font.height as f32;
			}
		}
		
		unsafe
		{
			gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
			gl::BufferData(
				gl::ARRAY_BUFFER,
				(vertices.len() * size_of::<f32>()) as isize,
				vertices.as_ptr() as *const _,
				gl::DYNAMIC_DRAW
			);

			gl::BindVertexArray(self.vao);
			gl::EnableVertexAttribArray(0);
			gl::EnableVertexAttribArray(1);

			gl::VertexAttribPointer(
				0,
				4,
				gl::FLOAT,
				gl::FALSE,
				(8 * size_of::<f32>()) as i32,
				std::ptr::null()
			);
			gl::VertexAttribPointer(
				1,
				4,
				gl::FLOAT,
				gl::FALSE,
				(8 * size_of::<f32>()) as i32,
				(4 * size_of::<f32>()) as *const _
			);
		}

		self.vertices = vertices.len() as i32 / 4;

		self.reload = false;
	}

	pub fn setSize(&mut self, size: u8)
	{
		self.fontSize = size;
		self.reload = true;
	}

	pub fn getBounds(&mut self) -> sdl2::rect::FRect
	{
		if self.reload { self.update(); }

		let p1 = self.ts.getMatrix() * glam::vec4(0.0, 0.0, 0.0, 1.0);
		let p2 = self.ts.getMatrix() * glam::vec4(self.dimensions.x, 0.0, 0.0, 1.0);
		let p3 = self.ts.getMatrix() * glam::vec4(self.dimensions.x, self.dimensions.y, 0.0, 1.0);
		let p4 = self.ts.getMatrix() * glam::vec4(0.0, self.dimensions.y, 0.0, 1.0);

		let min = p1.min(p2).min(p3).min(p4);
		let max = p1.max(p2).max(p3).max(p4);

		sdl2::rect::FRect::new(min.x, min.y, max.x - min.x, max.y - min.y)
	}

	pub fn getString(&mut self) -> String
	{
		let mut out = String::new();

		for part in &self.text
		{
			out += &part.text;
		}
		
		out
	}

	pub fn getDimensions(&mut self) -> glam::Vec2
	{
		if self.reload { self.update(); }
		self.dimensions
	}

	fn setPositionFN(_: &Lua, options: (f32, f32)) -> Result<(), Error>
	{
		let obj = unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() };
		obj.getText().ts.setPosition(glam::vec2(options.0, options.1));
		Ok(())
	}

	fn translateFN(_: &Lua, options: (f32, f32)) -> Result<(), Error>
	{
		let obj = unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() };
		obj.getText().ts.translate(glam::vec2(options.0, options.1));
		Ok(())
	}

	fn getPositionFN(_: &Lua, _: ()) -> Result<(Number, Number), Error>
	{
		let obj = unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() };
		Ok((
			obj.getText().ts.getPosition().x as f64,
			obj.getText().ts.getPosition().y as f64
		))
	}

	fn setRotationFN(_: &Lua, angle: f32) -> Result<(), Error>
	{
		let obj = unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() };
		obj.getText().ts.setRotation(angle);
		Ok(())
	}

	fn rotateFN(_: &Lua, angle: f32) -> Result<(), Error>
	{
		let obj = unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() };
		obj.getText().ts.rotate(angle);
		Ok(())
	}

	fn getRotationFN(_: &Lua, _: ()) -> Result<Number, Error>
	{
		let obj = unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() };
		Ok(obj.getText().ts.getRotation() as f64)
	}

	fn setScaleFN(_: &Lua, s: (f32, f32)) -> Result<(), Error>
	{
		let obj = unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() };
		obj.getText().ts.setScale(glam::vec2(s.0, s.1));
		Ok(())
	}

	fn scaleFN(_: &Lua, s: (f32, f32)) -> Result<(), Error>
	{
		let obj = unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() };
		obj.getText().ts.scale(glam::vec2(s.0, s.1));
		Ok(())
	}

	fn getScaleFN(_: &Lua, _: ()) -> Result<(Number, Number), Error>
	{
		let obj = unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() };
		Ok((
			obj.getText().ts.getScale().x as f64,
			obj.getText().ts.getScale().y as f64
		))
	}

	fn setOriginFN(_: &Lua, o: (f32, f32)) -> Result<(), Error>
	{
		let obj = unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() };
		obj.getText().ts.setOrigin(glam::vec2(o.0, o.1));
		Ok(())
	}

	fn getOriginFN(_: &Lua, _: ()) -> Result<(Number, Number), Error>
	{
		let obj = unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() };
		Ok((
			obj.getText().ts.getOrigin().x as f64,
			obj.getText().ts.getOrigin().y as f64
		))
	}

	fn boundsFN(_: &Lua, _: ()) -> Result<(Number, Number, Number, Number), Error>
	{
		let bounds = unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() }.getText().getBounds();
		Ok((
			bounds.left() as f64,
			bounds.top() as f64,
			bounds.width() as f64,
			bounds.height() as f64
		))
	}
	
	fn sizeFN(_: &Lua, _: ()) -> Result<(Number, Number), Error>
	{
		let size = unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() }.getText().getDimensions();
		Ok((size.x as f64, size.y as f64))
	}

	fn setStringFN(_: &Lua, txt: String) -> Result<(), Error>
	{
		let obj = unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() };
		obj.getText().setString(txt);
		Ok(())
	}

	fn getStringFN(_: &Lua, _: ()) -> Result<String, Error>
	{
		let obj = unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() };
		Ok(obj.getText().getString())
	}

	pub fn initLua(script: &Lua)
	{
		let table = script.create_table().unwrap();

		table.set("setPosition", script.create_function(Text::setPositionFN).unwrap());
		table.set("translate", script.create_function(Text::translateFN).unwrap());
		table.set("getPosition", script.create_function(Text::getPositionFN).unwrap());

		table.set("setRotation", script.create_function(Text::setRotationFN).unwrap());
		table.set("rotate", script.create_function(Text::rotateFN).unwrap());
		table.set("getRotation", script.create_function(Text::getRotationFN).unwrap());

		table.set("setScale", script.create_function(Text::setScaleFN).unwrap());
		table.set("scale", script.create_function(Text::scaleFN).unwrap());
		table.set("getScale", script.create_function(Text::getScaleFN).unwrap());

		table.set("setOrigin", script.create_function(Text::setOriginFN).unwrap());
		table.set("getOrigin", script.create_function(Text::getOriginFN).unwrap());

		table.set("bounds", script.create_function(Text::boundsFN).unwrap());
		table.set("size", script.create_function(Text::sizeFN).unwrap());

		table.set("setString", script.create_function(Text::setStringFN).unwrap());
		table.set("getString", script.create_function(Text::getStringFN).unwrap());
		
		script.globals().set("text", table);
	}
}

impl Drawable for Text
{
	fn draw(&mut self)
	{
		if self.reload { self.update(); }

		let shader = Window::getCamera().getTxtShader();
		unsafe
		{
			gl::BindVertexArray(self.vao);
			
			gl::ActiveTexture(gl::TEXTURE0);
			gl::BindTexture(gl::TEXTURE_2D, self.font.page);
			shader.activate();
			shader.setInt("tex", 0);
			shader.setMat4("model", &self.ts.getMatrix().to_cols_array());
			gl::DrawArrays(
				gl::QUADS,
				0,
				self.vertices
			);
			shader.setInt("layer", shader.getInt("layer") + 1);
		}
	}
}

pub struct Object
{
	name: String,
	image: Image,
	text: Text,
	children: Vec<Object>,
	script: Lua,
	init: bool,
	order: [char; 3],
	hasScript: bool,
	vars: Programmable
}

impl Object
{
	pub fn new() -> Self
	{
		Self
		{
			name: String::new(),
			image: Image::new(),
			text: Text::new(),
			children: vec![],
			script: Lua::new(),
			init: false,
			order: ['i', 't', 'c'],
			hasScript: false,
			vars: std::collections::HashMap::new()
		}
	}
	
	fn setNumFN(_: &Lua, options: (String, f32)) -> Result<(), Error>
	{
		let obj = unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() };
		obj.vars.insert(options.0, Variable { num: options.1, string: String::new() });
		Ok(())
	}
	
	fn getNumFN(_: &Lua, name: String) -> Result<Number, Error>
	{
		let obj = unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() };
		Ok(obj.vars.get(&name).unwrap_or(&Variable::new()).num as f64)
	}
	
	fn setStrFN(_: &Lua, options: (String, String)) -> Result<(), Error>
	{
		let obj = unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() };
		obj.vars.insert(options.0, Variable { num: 0.0, string: options.1 });
		Ok(())
	}
	
	fn getStrFN(_: &Lua, name: String) -> Result<String, Error>
	{
		let obj = unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() };
		Ok(obj.vars.get(&name).unwrap_or(&Variable::new()).string.clone())
	}

	fn nameFN(_: &Lua, _: ()) -> Result<String, Error>
	{
		let obj = unsafe { Window::getUI().scriptExecutor.as_mut().unwrap() };
		Ok(obj.name.clone())
	}

	pub fn parse(base: &spex::xml::Element) -> Self
	{
		let mut obj = Object::new();
		if base.name().local_part() != "object" { return obj; }

		obj.name = base.att_opt("name").unwrap_or_else(|| { println!("Object name not found"); "" }).to_string();

		let script = base.att_opt("script").unwrap_or("").to_string();
		if !script.is_empty()
		{
			obj.script.load_std_libs(StdLib::ALL_SAFE);
	
			Image::initLua(&obj.script);
			Text::initLua(&obj.script);
			Window::initLua(&obj.script);
			World::initLua(&obj.script);

			let table = obj.script.create_table().unwrap();

			table.set("setStr", obj.script.create_function(Object::setStrFN).unwrap());
			table.set("getStr", obj.script.create_function(Object::getStrFN).unwrap());
			table.set("setNum", obj.script.create_function(Object::setNumFN).unwrap());
			table.set("getNum", obj.script.create_function(Object::getNumFN).unwrap());
			table.set("name", obj.script.create_function(Object::nameFN).unwrap());

			obj.script.globals().set("object", table);

			obj.script.load(Assets::readFile(script).unwrap_or(String::new())).exec();

			obj.hasScript = true;
		}

		let order = base.att_opt("order").unwrap_or("");
		obj.order = [
			order.chars().nth(0).unwrap_or('i'),
			order.chars().nth(1).unwrap_or('t'),
			order.chars().nth(2).unwrap_or('c')
		];
		
		for node in base.elements()
		{
			let name = node.name().local_part();
			if name == "image" { obj.image = Image::parse(node); }
			if name == "text"
			{
				obj.text.loadFont(
					node.att_opt("font")
						.unwrap_or("")
						.to_string()
				);
				obj.text.setSize(
					node.att_opt("size")
						.unwrap_or("")
						.parse::<u8>()
						.unwrap_or(0)
				);
				obj.text.setString(
					node.text()
						.unwrap_or("")
						.to_string()
				);
			}
			if name == "object" { obj.children.push(Object::parse(node)); }
			if name == "var"
			{
				let name = node.att_opt("name").unwrap_or("").to_string();
				obj.vars.insert(name, Variable
				{
					num: node.att_opt("num").unwrap_or("0").parse::<f32>().unwrap(),
					string: node.att_opt("str").unwrap_or("").to_string()
				});
			}
		}
		
		obj
	}

	fn luaError(&mut self, res: Result<(), Error>)
	{
		if res.is_ok() { return; }
		println!("Object: {}\n{}\n", self.name, res.unwrap_err());
	}

	pub fn draw(&mut self)
	{
		crate::ae2d::Window::Window::getUI().scriptExecutor = self;
		if self.hasScript
		{
			if !self.init
			{
				self.init = true;
				self.luaError(self.script.globals().get::<Function>("Init").unwrap().call(()));
			}
			else
			{
				self.luaError(self.script.globals().get::<Function>("Update").unwrap().call(()));
			}
		}

		match self.order[0]
		{
			'i' => Window::getCamera().draw(&mut self.image),
			't' => Window::getCamera().draw(&mut self.text),
			'c' => for i in 0..self.children.len()
			{
				self.children[i].draw();
			}
			_ => {}
		}
		match self.order[1]
		{
			'i' => Window::getCamera().draw(&mut self.image),
			't' => Window::getCamera().draw(&mut self.text),
			'c' => for i in 0..self.children.len()
			{
				self.children[i].draw();
			}
			_ => {}
		}
		match self.order[2]
		{
			'i' => Window::getCamera().draw(&mut self.image),
			't' => Window::getCamera().draw(&mut self.text),
			'c' => for i in 0..self.children.len()
			{
				self.children[i].draw();
			}
			_ => {}
		}
	}

	pub fn getScript(&mut self) -> &Lua { &mut self.script }
	pub fn getText(&mut self) -> &mut Text { &mut self.text }
}

pub struct UI
{
	root: Object,
	projection: [f32; 16],
	pub scriptExecutor: *mut Object,
	view: Transformable2D,
	loadPath: String,
}

impl UI
{
	pub fn new() -> Self
	{
		Self
		{
			root: Object::new(),
			projection: glam::Mat4::IDENTITY.to_cols_array(),
			scriptExecutor: std::ptr::null::<Object>() as *mut _,
			view: Transformable2D::new(),
			loadPath: String::new(),
		}
	}

	pub fn fromFile(path: String) -> Self
	{
		let mut ui = UI::new();
		ui.load(path);
		ui
	}

	pub fn resize(&mut self)
	{
		let winSize = crate::ae2d::Window::Window::getSize();
		self.projection = glam::Mat4::orthographic_rh_gl(
			0.0,
			winSize.x,
			winSize.y,
			0.0,
			-100.0,
			100.0
		).to_cols_array();
	}

	pub fn load(&mut self, path: String)
	{
		let src = crate::ae2d::Assets::readXML(path);
		if src.is_none() { return; }

		self.root = Object::parse(&src.unwrap());
		self.view = Transformable2D::new();
		self.scriptExecutor = std::ptr::null::<Object>() as *mut Object;
	}

	pub fn render(&mut self)
	{
		if !self.loadPath.is_empty()
		{
			self.load(self.loadPath.clone());
			self.loadPath.clear();
		}
		Window::getCamera().toggleCameraTransform(false);
		self.root.draw();
	}

	pub fn requestReload(&mut self, path: String)
	{
		self.loadPath = path;
	}

	fn loadFileFN(_: &Lua, file: String) -> Result<(), Error>
	{
		Window::getUI().requestReload(file);
		Ok(())
	}
}