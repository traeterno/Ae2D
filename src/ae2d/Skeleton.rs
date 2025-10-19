use std::collections::HashMap;

use crate::ae2d::{Camera::Drawable, Shapes, Sprite::Sprite, Window::Window};

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

pub enum Interpolation
{
	Const,
	Linear,
	CubicIn, CubicOut, CubicInOut,
	SineIn, SineOut, SineInOut
}

impl ToString for Interpolation
{
	fn to_string(&self) -> String
	{
		match self
		{
			Interpolation::Const => "Const",
			Interpolation::Linear => "Linear",
			Interpolation::CubicIn => "CubicIn",
			Interpolation::CubicOut => "CubicOut",
			Interpolation::CubicInOut => "CubicInOut",
			Interpolation::SineIn => "SineIn",
			Interpolation::SineOut => "SineOut",
			Interpolation::SineInOut => "SineInOut",
		}.to_string()
	}
}

pub struct Frame
{
	pub timestamp: f32,
	pub angle: (Interpolation, f32),
	pub texture: String
}

impl Frame
{
	pub fn new() -> Self
	{
		Self
		{
			timestamp: 0.0,
			angle: (Interpolation::Const, 0.0),
			texture: String::new()
		}
	}

	pub fn parse(node: &json::JsonValue, ts: f32) -> Self
	{
		let mut f = Self::new();
		f.timestamp = ts;
		for (var, value) in node.entries()
		{
			if var == "angle"
			{
				let a = value.as_str()
					.unwrap_or("").split(" ")
					.collect::<Vec<&str>>();
				let angle = a[1].parse::<f32>().unwrap_or(0.0);
				f.angle = (match a[0]
				{
					"Linear" => Interpolation::Linear,
					"CubicIn" => Interpolation::CubicIn,
					"CubicOut" => Interpolation::CubicOut,
					"CubicInOut" => Interpolation::CubicInOut,
					"SineIn" => Interpolation::SineIn,
					"SineOut" => Interpolation::SineOut,
					"SineInOut" => Interpolation::SineInOut,
					_ => Interpolation::Const
				}, angle);
			}
			if var == "texture"
			{
				f.texture = value.as_str().unwrap_or("").to_string();
			}
		}
		f
	}
}

pub struct Timeline
{
	pub frames: Vec<Frame>,
	pub current: usize
}

impl Timeline
{
	pub fn new() -> Self
	{
		Self
		{
			frames: vec![],
			current: 0
		}
	}

	pub fn parse(node: &json::JsonValue) -> Self
	{
		let mut tl = Self::new();
		for (point, frame) in node.entries()
		{
			tl.frames.push(Frame::parse(
				frame,
				point.parse().unwrap()
			));
		}
		tl
	}

	pub fn update(&mut self, bone: &mut Bone, repeat: bool, time: f32)
	{
		if self.frames.len() == 0 { return; }
		if self.current == self.frames.len() - 1 && self.frames.len() > 1
		{
			if !repeat { return; }
			if self.frames[self.current].timestamp > time { self.current = 0; }
			else { return; }
		}

		if self.frames.len() == 1
		{
			let f = &self.frames[0];
			bone.angle = f.angle.1;
			if !f.texture.is_empty() { bone.texture = f.texture.clone(); }
			return;
		}

		let start = &self.frames[self.current];
		let end = &self.frames[self.current + 1];

		let ct = time - start.timestamp;
		let t = ct / (end.timestamp - start.timestamp);
		let a = end.angle.1 - start.angle.1;

		if !start.texture.is_empty() { bone.texture = start.texture.clone(); }

		bone.angle = start.angle.1 + a * match start.angle.0
		{
			Interpolation::Const => 0.0,
			Interpolation::Linear => t,
			Interpolation::CubicIn => t.powi(3),
			Interpolation::CubicOut => 1.0 - (t - 1.0).powi(3),
			Interpolation::CubicInOut =>
				if t < 0.5 { 4.0 * t.powi(3) }
				else { 1.0 - (-2.0 * t + 2.0).powi(3) / 2.0 },
			Interpolation::SineIn => 1.0 - (t * std::f32::consts::PI / 2.0).cos(),
			Interpolation::SineOut => (t * std::f32::consts::PI / 2.0).sin(),
			Interpolation::SineInOut => -((t * std::f32::consts::PI).cos() - 1.0) / 2.0
		};

		if time >= end.timestamp { self.current += 1; }
	}
}

pub struct Animation
{
	pub repeat: bool,
	pub bones: HashMap<String, Timeline>,
	pub time: f32,
	pub duration: f32
}

impl Animation
{
	pub fn new() -> Self
	{
		Self
		{
			repeat: false,
			bones: HashMap::new(),
			time: 0.0,
			duration: 0.0
		}
	}

	pub fn parse(node: &json::JsonValue) -> Self
	{
		let mut anim = Self::new();
		for (section, data) in node.entries()
		{
			if section == "repeat" { anim.repeat = data.as_bool().unwrap(); }
			if section == "bones"
			{
				for (path, frames) in data.entries()
				{
					anim.bones.insert(
						path.to_string(),
						Timeline::parse(frames)
					);
				}
			}
		}
		anim.calculateDuration();
		anim
	}

	pub fn update(&mut self, root: &mut Bone, progress: bool)
	{
		for (bone, timeline) in &mut self.bones
		{
			let path = bone.split("/").collect::<Vec<&str>>();
			if let Some(bone) = root.resolvePath(path)
			{
				timeline.update(bone, self.repeat, self.time);
			}
		}
		if progress { self.time += Window::getDeltaTime(); }
		if self.time > self.duration { self.restart(); }
	}

	pub fn restart(&mut self)
	{
		self.time = 0.0;
		for (_, tl) in &mut self.bones { tl.current = 0; }
	}

	pub fn calculateDuration(&mut self)
	{
		self.duration = 0.0;
		for (_, tl) in &self.bones
		{
			if let Some(f) = tl.frames.last()
			{
				self.duration = self.duration.max(f.timestamp);
			}
		}
	}
}

pub struct Skeleton
{
	root: Bone,
	sprites: SpriteList,
	visible: Sprite,
	anims: HashMap<String, Animation>,
	currentAnim: String,
	pub debug: bool,
	pub activeAnim: bool
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
			anims: HashMap::new(),
			currentAnim: String::new(),
			debug: true,
			activeAnim: true
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
			self.sprites.clear();
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

	pub fn loadAL(&mut self, path: String)
	{
		println!("Loading animation list from {path}...");
		let raw = std::fs::read_to_string(path);
		if raw.is_err() { return; }
		if let Ok(root) = json::parse(&raw.unwrap())
		{
			if root.len() == 0 { return; }
			self.anims.clear();
			for (name, value) in root.entries()
			{
				self.anims.insert(
					name.to_string(),
					Animation::parse(value)
				);
			}
		}
	}

	pub fn update(&mut self) { self.root.update(glam::Vec2::ZERO, 0.0); }

	pub fn getRoot(&mut self) -> &mut Bone { &mut self.root }

	pub fn getSL(&mut self) -> &mut SpriteList { &mut self.sprites }

	pub fn loadTexture(&mut self, path: String)
	{
		self.visible = Sprite::image(path);
	}

	pub fn setAnimation(&mut self, anim: String)
	{
		if self.currentAnim == anim { return; }
		if !self.anims.contains_key(&anim) { return; }
		if let Some(a) = self.anims.get_mut(&self.currentAnim)
		{
			a.restart();
		}
		self.currentAnim = anim;
	}

	pub fn getCurrentAnimation(&mut self) -> (String, Option<&mut Animation>)
	{
		(self.currentAnim.clone(), self.anims.get_mut(&self.currentAnim))
	}

	pub fn getAnimations(&mut self) -> &mut HashMap<String, Animation>
	{
		&mut self.anims
	}
}

impl Drawable for Skeleton
{
	fn draw(&mut self)
	{
		if let Some(a) = self.anims.get_mut(&self.currentAnim)
		{
			a.update(&mut self.root, self.activeAnim);
		}
		
		for i in 0..10
		{
			self.root.draw(&mut self.visible, &self.sprites, i);
		}
		if self.debug { self.root.drawDebug(); }
	}
}