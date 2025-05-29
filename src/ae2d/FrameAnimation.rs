use sdl2::rect::FRect;

#[derive(Clone, Debug)]
pub struct Frame
{
	id: u8,
	duration: f32
}

#[derive(Clone, Debug)]
pub struct Animation
{
	name: String,
	repeat: i32,
	repeated: i32,
	frames: Vec<Frame>,
	currentTime: f32,
	currentFrame: usize
}

impl Animation
{
	pub fn new() -> Self
	{
		Self
		{
			name: String::new(),
			repeat: 0,
			repeated: 0,
			frames: vec![],
			currentFrame: 0,
			currentTime: 0.0
		}
	}

	pub fn oneFrame(name: String, id: u8) -> Self
	{
		Self
		{
			name,
			repeat: 0,
			repeated: 0,
			frames: vec![Frame { duration: 0.0, id }],
			currentFrame: 0,
			currentTime: 0.0
		}
	}

	pub fn parse(base: &spex::xml::Element) -> Self
	{
		let mut anim = Animation::new();

		anim.name = base.att_opt("name").unwrap_or("").to_string();
		anim.repeat = base.att_opt("repeat").unwrap_or("0").parse().unwrap();

		for frame in base.elements()
		{
			anim.frames.push(Frame {
				id: frame.att_opt("id").unwrap_or("0").parse().unwrap(),
				duration: frame.att_opt("duration").unwrap_or("0").parse().unwrap()
			});
		}
		
		anim
	}

	pub fn update(&mut self) -> bool
	{
		if self.repeated >= self.repeat && self.repeat != 0 { return false; }
		let prev = if self.currentTime == 0.0 {usize::MAX} else {self.currentFrame};

		self.currentTime += crate::ae2d::Window::Window::getDeltaTime();
		if self.currentTime >= self.frames[self.currentFrame].duration
		{
			self.currentTime -= self.frames[self.currentFrame].duration;
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
		return self.currentFrame != prev;
	}

	pub fn getCurrentFrame(&mut self) -> u8 { self.frames[self.currentFrame].id }
}

#[derive(Clone, Debug)]
pub struct Animator
{
	texture: u32,
	size: glam::IVec2,
	frame: glam::IVec2,
	animations: Vec<Animation>,
	currentAnimation: usize,
	frames: Vec<FRect>
}

impl Animator
{
	pub fn new() -> Self
	{
		Self
		{
			texture: 0,
			size: glam::IVec2::splat(0),
			frame: glam::IVec2::splat(0),
			animations: vec![],
			currentAnimation: 0,
			frames: vec![]
		}
	}

	pub fn load(&mut self, path: String)
	{
		let src = crate::ae2d::Assets::readXML(path);
		if src.is_none() { return; }
		let doc = src.unwrap();

		self.texture = crate::ae2d::Assets::getTexture(
			doc.att_opt("texture")
				.unwrap_or("")
				.to_string()
		);
		unsafe
		{
			gl::BindTexture(gl::TEXTURE_2D, self.texture);
			gl::GetTexLevelParameteriv(gl::TEXTURE_2D, 0, gl::TEXTURE_WIDTH, &mut self.size.x);
			gl::GetTexLevelParameteriv(gl::TEXTURE_2D, 0, gl::TEXTURE_HEIGHT, &mut self.size.y);
			gl::BindTexture(gl::TEXTURE_2D, 0);
		}
		self.frame.x = doc.att_opt("w").unwrap_or("1").parse().unwrap();
		self.frame.y = doc.att_opt("h").unwrap_or("1").parse().unwrap();
		self.calculateFrames();

		for node in doc.elements()
		{
			self.animations.push(Animation::parse(node));
		}
	}

	pub fn fromFile(path: String) -> Self
	{
		let mut anim = Animator::new();
		anim.load(path);
		anim
	}

	pub fn bindTexture(&mut self)
	{
		unsafe
		{
			gl::BindTexture(gl::TEXTURE_2D, self.texture);
		}
	}

	pub fn getSize(&mut self) -> glam::IVec2 { self.size }

	pub fn getFrameSize(&mut self) -> glam::IVec2 { self.frame }

	pub fn update(&mut self) -> bool
	{
		if self.animations.len() == 0 { return false; }
		self.animations[self.currentAnimation].update()
	}

	fn calculateFrames(&mut self)
	{
		self.frames.clear();
		let mut x = 0;
		let mut y = 0;
		while y < self.size.y
		{
			while x < self.size.x
			{
				self.frames.push(sdl2::rect::FRect::new(
					x as f32 / self.size.x as f32,
					y as f32 / self.size.y as f32,
					self.frame.x as f32 / self.size.x as f32,
					self.frame.y as f32 / self.size.y as f32
				));
				x += self.frame.x;
			}
			y += self.frame.y;
			x = 0;
		}
	}

	pub fn getCurrentFrame(&mut self) -> sdl2::rect::FRect
	{
		if self.frames.len() == 0 { return sdl2::rect::FRect::new(0.0 ,0.0, 0.0, 0.0); }
		if self.animations.len() == 0 { return sdl2::rect::FRect::new(0.0, 0.0, 0.0, 0.0); }
		self.frames[self.animations[self.currentAnimation].getCurrentFrame() as usize]
	}

	pub fn setCurrentAnimation(&mut self, name: String)
	{
		for i in 0..self.animations.len()
		{
			if self.animations[i].name == name && self.currentAnimation != i
			{
				self.currentAnimation = i;
				self.restart();
				return;
			}
		}
	}

	fn restart(&mut self)
	{
		for i in 0..self.animations.len()
		{
			self.animations[i].currentTime = 0.0;
			self.animations[i].currentFrame = 0;
		}
	}
}