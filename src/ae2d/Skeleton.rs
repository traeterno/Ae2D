use std::collections::HashMap;

use crate::ae2d::{Camera::Drawable, Shapes, Sprite::Sprite};

pub struct Bone
{
	angle: f32,
	length: f32,
	texture: String,
	children: HashMap<String, Bone>,
	pos: glam::Vec2,
	parentAngle: f32,
}

impl Bone
{
	pub fn new() -> Self
	{
		Self
		{
			angle: 0.0,
			length: 0.0,
			texture: String::new(),
			children: HashMap::new(),
			pos: glam::Vec2::ZERO,
			parentAngle: 0.0,
		}
	}

	pub fn parse(obj: &json::JsonValue) -> Self
	{
		let mut s = Self::new();
		for (var, value) in obj.entries()
		{
			if var == "angle" { s.angle = value.as_f32().unwrap() }
			if var == "length" { s.length = value.as_f32().unwrap(); }
			if var == "texture" { s.texture = value.as_str().unwrap().to_string(); }
			if var == "children"
			{
				for (id, data) in value.entries()
				{
					s.children.insert(
						id.to_string(),
						Bone::parse(data)
					);
				}
			}
		}
		s
	}

	pub fn getEnd(&self) -> glam::Vec2
	{
		let a = (90.0 + self.parentAngle + self.angle).to_radians();
		self.pos + glam::vec2(
			a.cos() * self.length,
			a.sin() * self.length
		)
	}

	pub fn update(&mut self, pos: glam::Vec2, angle: f32)
	{
		self.parentAngle = angle;
		self.pos = pos;
		let p = self.getEnd();
		for (_, b) in &mut self.children
		{
			b.update(p, self.angle + angle);
		}
	}

	fn draw(&mut self, spr: &mut Sprite, sl: &HashMap<String, (glam::Vec4, glam::Vec2)>)
	{
		if !self.texture.is_empty()
		{
			if let Some((r, os)) = sl.get(&self.texture)
			{
				spr.getTransformable().setOrigin(*os);
				spr.setTextureRect(*r);
				spr.getTransformable().setPosition(self.pos);
				spr.getTransformable().setRotation(self.parentAngle + self.angle - 180.0);
				spr.draw();
			}
		}
		Shapes::line(
			self.pos,
			self.getEnd(),
			glam::vec4(1.0, 0.0, 0.0, 1.0),
			glam::vec4(0.0, 0.0, 1.0, 1.0)
		);
		for (_, b) in &mut self.children
		{
			b.draw(spr, sl);
		}
	}
}

pub struct Skeleton
{
	root: Bone,
	sprites: HashMap<String, (glam::Vec4, glam::Vec2)>,
	visible: Sprite
}

impl Skeleton
{
	pub fn new() -> Self
	{
		Self
		{
			root: Bone::new(),
			sprites: HashMap::new(),
			visible: Sprite::default()
		}
	}

	pub fn loadRig(&mut self, path: String)
	{
		println!("Loading rig from \"{path}\"...");
		let node = json::parse(
			&std::fs::read_to_string(path).unwrap()
		);
		if let Ok(root) = node
		{
			self.root = Bone::parse(root.entries().nth(0).unwrap().1);
		}
	}

	pub fn loadSprites(&mut self, path: String)
	{
		println!("Loading sprite list from \"{path}\"...");
		let node = json::parse(
			&std::fs::read_to_string(path).unwrap()
		);
		if let Ok(root) = node
		{
			for (var, value) in root.entries()
			{
				if var == "texture"
				{
					self.visible = Sprite::image(value.as_str().unwrap().to_string());
					println!("Loading texture from {}", value.as_str().unwrap());
				}
				if var == "sprites"
				{
					for (id, data) in value.entries()
					{
						let mut os = glam::Vec2::ZERO;
						let mut r = glam::Vec4::ZERO;
						for (x, y) in data.entries()
						{
							let z = y
								.members().map(
									|a| a.as_f32().unwrap()
								).collect::<Vec<f32>>();
							if x == "offset"
							{
								os = glam::vec2(z[0], z[1]);
							}
							if x == "rect"
							{
								r = glam::vec4(z[0], z[1], z[2], z[3]);
							}
						}
						self.sprites.insert(
							id.to_string(),
							(r, os)
						);
					}
					println!("Found {} sprites", self.sprites.len());
				}
			}
		}
	}

	pub fn update(&mut self) { self.root.update(glam::Vec2::ZERO, 0.0); }
}

impl Drawable for Skeleton
{
	fn draw(&mut self)
	{
		self.root.draw(&mut self.visible, &self.sprites);
	}
}