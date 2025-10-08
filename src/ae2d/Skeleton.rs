use std::collections::HashMap;

use crate::ae2d::{Camera::Drawable, Shapes, Sprite::Sprite};

pub type SpriteList = HashMap<String, (glam::Vec4, glam::Vec2)>;

pub struct Bone
{
	pub angle: f32,
	pub length: f32,
	pub texture: String,
	pub layer: u8,
	children: HashMap<String, Bone>,
	pos: glam::Vec2,
	parentAngle: f32,

	pub highlight: bool
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
			layer: 0,
			children: HashMap::new(),
			pos: glam::Vec2::ZERO,
			parentAngle: 0.0,

			highlight: false
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
			if var == "layer" { s.layer = value.as_u8().unwrap(); }
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

	pub fn getBone(&mut self, name: String) -> Option<&mut Bone>
	{
		self.children.get_mut(&name)
	}

	pub fn getBones(&mut self) -> &mut HashMap<String, Bone>
	{
		&mut self.children
	}

	pub fn serialize(&self, s: &mlua::Lua) -> mlua::Table
	{
		let t = s.create_table().unwrap();
		for (id, b) in &self.children
		{
			let _ = t.raw_set(id.clone(), b.serialize(s));
		}
		t
	}

	pub fn resolvePath(&mut self, mut path: Vec<&str>) -> Option<&mut Bone>
	{
		if path.len() == 0 { return Some(self); }
		for (name, b) in &mut self.children
		{
			if name == path[0]
			{
				path.remove(0);
				return b.resolvePath(path);
			}
		}
		None
	}

	pub fn draw(&mut self, spr: &mut Sprite, sl: &SpriteList, layer: u8)
	{
		if !self.texture.is_empty() && self.layer == layer
		{
			if let Some((r, os)) = sl.get(&self.texture)
			{
				spr.getTransformable().setOrigin(*os);
				spr.setTextureRect(*r);
				spr.getTransformable().setPosition(self.pos);
				spr.getTransformable().setRotation(self.parentAngle + self.angle);
				if self.highlight { spr.setColor((200, 200, 200, 255)); }
				else { spr.setColor((255, 255, 255, 255)); }
				spr.draw();
			}
			self.highlight = false;
		}
		for (_, b) in &mut self.children
		{
			b.draw(spr, sl, layer);
		}
	}

	pub fn drawDebug(&mut self)
	{
		Shapes::line(
			self.pos,
			self.getEnd(),
			glam::vec4(1.0, 0.0, 0.0, 1.0),
			glam::vec4(0.0, 0.0, 1.0, 1.0)
		);
		for (_, b) in &mut self.children { b.drawDebug(); }
	}

	pub fn toJSON(&self) -> json::JsonValue
	{
		let mut children = json::object!{};
		for (name, data) in &self.children
		{
			let _ = children.insert(name.as_str(), data.toJSON());
		}
		json::object!{
			angle: self.angle,
			length: self.length,
			texture: self.texture.clone(),
			layer: self.layer,
			children: children
		}
	}
}

pub struct Skeleton
{
	root: Bone,
	sprites: SpriteList,
	visible: Sprite,
	pub debug: bool
}

impl Skeleton
{
	pub fn new() -> Self
	{
		Self
		{
			root: Bone::new(),
			sprites: HashMap::new(),
			visible: Sprite::default(),
			debug: true
		}
	}

	pub fn loadRig(&mut self, path: String)
	{
		println!("Loading rig from \"{path}\"...");
		let raw = std::fs::read_to_string(path);
		if let Ok(f) = raw
		{
			if let Ok(root) = json::parse(&f)
			{
				if root.len() == 0 { return; }
				self.root = Bone::parse(root.entries().nth(0).unwrap().1);
			}
		}
	}

	pub fn loadSL(&mut self, path: String) -> String
	{
		println!("Loading sprite list from \"{path}\"...");
		let raw = std::fs::read_to_string(path);
		let mut texPath = String::new();
		if raw.is_err() { return texPath; }
		if let Ok(root) = json::parse(&raw.unwrap())
		{
			for (var, value) in root.entries()
			{
				if var == "texture"
				{
					texPath = value.as_str().unwrap().to_string();
					texPath = texPath.replace("\\", "/");
					self.visible = Sprite::image(texPath.clone());
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
		texPath
	}

	pub fn update(&mut self) { self.root.update(glam::Vec2::ZERO, 0.0); }

	pub fn getRoot(&mut self) -> &mut Bone { &mut self.root }

	pub fn getSL(&mut self) -> &mut SpriteList { &mut self.sprites }

	pub fn loadTexture(&mut self, path: String)
	{
		self.visible = Sprite::image(path);
	}
}

impl Drawable for Skeleton
{
	fn draw(&mut self)
	{
		for i in 0..=5
		{
			self.root.draw(&mut self.visible, &self.sprites, i);
		}
		if self.debug { self.root.drawDebug(); }
	}
}