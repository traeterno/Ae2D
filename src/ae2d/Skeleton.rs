use std::{fs::File, io::Read};

use spex::parsing::XmlReader;

use super::{envell::Sprite::Sprite, Camera::Drawable, Window::Window};

#[derive(Debug)]
struct Texture
{
	name: String,
	origin: glam::Vec2,
	rect: glam::Vec4,
}

#[derive(Debug)]
struct Visible
{
	scale: glam::Vec2,
	layer: u8,
	texture: u8,
	transform: glam::Vec3,
}

#[derive(Debug)]
struct Bone
{
	parentAngle: f32,
	angle: f32,
	length: f32,
	visible: i8,
	parent: i8,
	pos: glam::Vec2,
	scale: f32
}

#[derive(Debug)]
struct Animation
{
	name: String,
	duration: f32,
	repeat: bool,
	mainBone: i8,
	changes: Vec<Change>,
	currentTime: f32
}

#[derive(Debug)]
struct Frame
{
	duration: f32,
	angle: f32,
	boneScale: f32,
	scale: glam::Vec2
}

#[derive(Debug)]
struct Change
{
	boneID: u8,
	frames: Vec<Frame>,
	currentFrame: u8,
	currentTime: f32
}

pub struct Skeleton
{
	spr: Sprite,
	textures: Vec<Texture>,
	vBones: Vec<Visible>,
	bones: Vec<Bone>,
	anims: Vec<Animation>,
	visible: bool,
	currentAnim: i8,
	mainBone: u8,
	posOffset: glam::Vec2,
	position: glam::Vec2,
	scale: f32
}

impl Skeleton
{
	pub fn new() -> Self
	{
		Self
		{
			spr: Sprite::new(),
			textures: vec![],
			vBones: vec![],
			bones: vec![],
			anims: vec![],
			visible: false,
			currentAnim: -1,
			mainBone: 0,
			posOffset: glam::Vec2::ZERO,
			position: glam::Vec2::ZERO,
			scale: 1.0
		}
	}

	pub fn load(path: String) -> Self
	{
		let buffer = &mut [0u8; u16::MAX as usize];
		let bufferSize = File::open(path).unwrap().read(buffer).unwrap();
		let texSize = u64::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6], buffer[7]]);

		let mut spr = Sprite::new();
		match stb_image::image::load_from_memory(&buffer[8..(8 + texSize as usize)])
		{
			stb_image::image::LoadResult::Error(x) => { println!("Error occured: {x}"); spr = Sprite::new() }
			stb_image::image::LoadResult::ImageF32(_) => { println!("Opened ImageF32; not supported"); spr = Sprite::new(); }
			stb_image::image::LoadResult::ImageU8(img) =>
			{
				spr.setTexture(img);
			}
		};

		let doc = XmlReader::parse_auto(&buffer[(8 + texSize as usize)..(bufferSize)]).unwrap();

		let mut textures: Vec<Texture> = vec![];
		let mut vBones: Vec<Visible> = vec![];
		let mut bones: Vec<Bone> = vec![];
		let mut anims: Vec<Animation> = vec![];

		for node in doc.root().elements()
		{
			let name = node.name().local_part();
			if name == "texture"
			{
				let texName = node.att_opt("name").unwrap_or("").to_string();
				let originRaw = node.att_opt("origin")
					.unwrap_or("0 0").to_string();
				let rectRaw= node.att_opt("rect")
					.unwrap_or("0 0 0 0").to_string();

				let o: Vec<&str> = originRaw.split(" ").collect();
				let r: Vec<&str> = rectRaw.split(" ").collect();

				
				let origin = glam::vec2(o[0].parse().unwrap(), o[1].parse().unwrap());
				let rect = glam::vec4(
					r[0].parse::<f32>().unwrap(), r[1].parse::<f32>().unwrap(),
					r[2].parse::<f32>().unwrap(), r[3].parse::<f32>().unwrap()
				);
				
				textures.push(Texture {
					name: texName,
					origin, rect
				});
			}
			else if name == "visible"
			{
				vBones.push(Visible {
					layer: node.att_opt("layer").unwrap_or("0").parse().unwrap(),
					scale: glam::vec2(
						node.att_opt("scaleX").unwrap_or("1").parse().unwrap(),
						node.att_opt("scaleY").unwrap_or("1").parse().unwrap()
					),
					texture: node.att_opt("texture").unwrap_or("0").parse().unwrap(),
					transform: glam::Vec3::ZERO
				});
			}
			else if name == "bone"
			{
				bones.push(Bone {
					parentAngle: 0.0,
					angle: node.att_opt("angle").unwrap_or("0").parse().unwrap(),
					length: node.att_opt("length").unwrap_or("0").parse().unwrap(),
					visible: node.att_opt("visible").unwrap_or("0").parse().unwrap(),
					parent: node.att_opt("parent").unwrap_or("-1").parse().unwrap(),
					pos: glam::Vec2::ZERO,
					scale: 1.0
				});
			}
			else if name == "animation"
			{
				let mut anim = Animation {
					name: String::from(node.att_opt("name").unwrap_or("")),
					repeat: node.att_opt("repeat").unwrap_or("false").parse().unwrap(),
					duration: node.att_opt("duration").unwrap_or("0").parse().unwrap(),
					mainBone: node.att_opt("mainBone").unwrap_or("0").parse().unwrap(),
					changes: vec![],
					currentTime: 0.0
				};

				for c in node.elements()
				{
					let mut change = Change {
						boneID: c.att_opt("id").unwrap_or("0").parse().unwrap(),
						frames: vec![],
						currentFrame: 0,
						currentTime: 0.0
					};

					for f in c.elements()
					{
						change.frames.push(Frame {
							duration: f.att_opt("duration").unwrap_or("0").parse().unwrap(),
							angle: f.att_opt("angle").unwrap_or("0").parse().unwrap(),
							boneScale: f.att_opt("boneScale").unwrap_or("1").parse().unwrap(),
							scale: glam::vec2(
								f.att_opt("scaleX").unwrap_or("1").parse().unwrap(),
								f.att_opt("scaleY").unwrap_or("1").parse().unwrap()
							)
						});
					}

					if change.frames.len() == 0 { continue; }

					anim.changes.push(change);
				}
				
				anims.push(anim);
			}
		}

		Self
		{
			spr, textures, vBones, bones, anims,
			visible: true,
			currentAnim: -1,
			mainBone: 0,
			posOffset: glam::Vec2::ZERO,
			position: glam::Vec2::ZERO,
			scale: 1.0
		}
	}

	pub fn setAnimation(&mut self, name: String)
	{
		for i in 0..self.anims.len()
		{
			if self.anims[i].name == name
			{
				self.posOffset = glam::Vec2::ZERO;
				self.currentAnim = i as i8;
				break;
			}
		}
	}

	pub fn update(&mut self)
	{
		if !self.visible { return; }

		if self.currentAnim != -1
		{
			let anim = &mut self.anims[self.currentAnim as usize];
			if anim.mainBone != -1 { self.mainBone = anim.mainBone as u8; }
			anim.currentTime += Window::getDeltaTime();
			if anim.currentTime > anim.duration
			{
				if anim.repeat { anim.currentTime = 0.0; }
				else { self.currentAnim = -1; }
			}

			for c in &mut anim.changes
			{
				c.currentTime += Window::getDeltaTime();

				let bone = &mut self.bones[c.boneID as usize];
				
				let mut cf = &c.frames[c.currentFrame as usize];
				if c.currentTime > cf.duration
				{
					c.currentFrame += 1;
					c.currentTime = 0.0;
					if c.currentFrame == c.frames.len() as u8 { c.currentFrame = 0; }
					cf = &c.frames[c.currentFrame as usize];
				}

				let nf =
					if c.currentFrame + 1 == c.frames.len() as u8 { &c.frames[0] }
					else { &c.frames[c.currentFrame as usize + 1] };

				let delta = c.currentTime / cf.duration;

				bone.angle = cf.angle + (nf.angle - cf.angle) * delta;
				bone.scale = cf.boneScale + (nf.boneScale - cf.boneScale) * delta;

				if bone.visible == -1 { continue; }

				let vb = &mut self.vBones[bone.visible as usize];
				vb.scale = cf.scale + (nf.scale - cf.scale) * delta;
			}
		}

		let mainBonePos = self.bones[self.mainBone as usize].pos;

		for i in 0..self.bones.len()
		{
			let parentID = self.bones[i].parent;
			let mut pos = glam::Vec2::ZERO;
			let mut pa = 0.0;
			if parentID != -1
			{
				let bone = &self.bones[parentID as usize];
				
				pa = bone.parentAngle + bone.angle;
				let r = (pa + 90.0).to_radians();
				pos = bone.pos + (glam::vec2(r.cos(), r.sin()) * bone.length * bone.scale * self.scale);
			}
			
			let bone = &mut self.bones[i];
			bone.pos = pos;
			bone.parentAngle = pa;
		}
		self.posOffset += mainBonePos - self.bones[self.mainBone as usize].pos;

		for bone in &self.bones
		{
			if bone.visible == -1 { continue; }
			let visible = &mut self.vBones[bone.visible as usize];
			visible.transform = glam::vec3(
				bone.pos.x, bone.pos.y,
				bone.parentAngle + bone.angle
			);
		}
	}

	pub fn setPosition(&mut self, pos: glam::Vec2) { self.position = pos; }
	pub fn setScale(&mut self, scale: f32) { self.scale = scale; }
}

impl Drawable for Skeleton
{
	fn draw(&mut self)
	{
		if !self.visible { return; }
		
		for layer in 0..5
		{
			for visible in &self.vBones
			{
				if visible.layer != layer { continue; }
				
				let t = &self.textures[visible.texture as usize];

				self.spr.setTextureRect(t.rect);
				self.spr.getTransform().setOrigin(t.origin);
				self.spr.getTransform().setScale(visible.scale * self.scale);
				self.spr.getTransform().setPosition(glam::vec2(
					visible.transform.x,
					visible.transform.y
				) + self.posOffset + self.position);
				self.spr.getTransform().setRotation(visible.transform.z);
				self.spr.draw();
			}
		}
	}
}