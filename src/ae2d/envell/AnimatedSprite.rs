use mlua::{Error, Lua};

use crate::ae2d::{Camera::Drawable, FrameAnimation::Animator, Transformable::Transformable2D, Window::Window};

#[derive(Clone, Debug)]
pub struct AnimatedSprite
{
	anim: Animator,
	vao: u32,
	vbo: u32,
	ts: Transformable2D,
	color: glam::Vec4,
}

impl AnimatedSprite
{
	pub fn new() -> Self
	{
		Self
		{
			anim: Animator::new(),
			vao: 0,
			vbo: 0,
			ts: Transformable2D::new(),
			color: glam::Vec4::ONE,
		}
	}

	pub fn loadAnimator(&mut self, path: String) { self.anim.load(path); }
	pub fn getTransform(&mut self) -> &mut Transformable2D { &mut self.ts }

	fn update(&mut self)
	{
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

		if self.anim.update()
		{
			let frame = self.anim.getCurrentFrame();
			let size = self.anim.getFrameSize();
			let vertices = [
				0.0, 0.0, frame.x, frame.y,
				self.color.x, self.color.y, self.color.z, self.color.w,
	
				size.x as f32, 0.0, frame.x + frame.w, frame.y,
				self.color.x, self.color.y, self.color.z, self.color.w,
				
				size.x as f32, size.y as f32, frame.x + frame.w, frame.y + frame.h,
				self.color.x, self.color.y, self.color.z, self.color.w,
				
				0.0, size.y as f32, frame.x, frame.y + frame.h,
				self.color.x, self.color.y, self.color.z, self.color.w
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
	}

	pub fn initLua(script: &Lua)
	{
		let table = script.create_table().unwrap();

		table.set("setAnimation", script.create_function(AnimatedSprite::setAnimFN).unwrap());
		table.set("setColor", script.create_function(AnimatedSprite::setColorFN).unwrap());
		table.set("draw", script.create_function(AnimatedSprite::drawFN).unwrap());
		table.set("bounds", script.create_function(AnimatedSprite::boundsFN).unwrap());
		table.set("setPosition", script.create_function(AnimatedSprite::setPosFN).unwrap());
		table.set("translate", script.create_function(AnimatedSprite::addPosFN).unwrap());
		table.set("getPosition", script.create_function(AnimatedSprite::getPosFN).unwrap());
		table.set("setRotation", script.create_function(AnimatedSprite::setRotationFN).unwrap());
		table.set("rotate", script.create_function(AnimatedSprite::addRotationFN).unwrap());
		table.set("getRotation", script.create_function(AnimatedSprite::getRotationFN).unwrap());
		table.set("setScale", script.create_function(AnimatedSprite::setScaleFN).unwrap());
		table.set("scale", script.create_function(AnimatedSprite::addScaleFN).unwrap());
		table.set("getScale", script.create_function(AnimatedSprite::getScaleFN).unwrap());
		table.set("setOrigin", script.create_function(AnimatedSprite::setOriginFN).unwrap());
		table.set("getOrigin", script.create_function(AnimatedSprite::getOriginFN).unwrap());

		script.globals().set("sprite", table);
	}

	pub fn setAnimFN(_: &Lua, data: (usize, String)) -> Result<(), Error>
	{
		Window::getWorld().getCurrentEntity().getSprite(data.0).anim.setCurrentAnimation(data.1); Ok(())
	}

	pub fn setColorFN(_: &Lua, data: (usize, u8, u8, u8, u8)) -> Result<(), Error>
	{
		Window::getWorld().getCurrentEntity().getSprite(data.0).color = glam::vec4(
			data.1 as f32 / 255.0,
			data.2 as f32 / 255.0,
			data.3 as f32 / 255.0,
			data.4 as f32 / 255.0
		);
		Ok(())
	}
	
	pub fn drawFN(_: &Lua, id: usize) -> Result<(), Error>
	{
		Window::getWorld().getCurrentEntity().getSprite(id).draw(); Ok(())
	}

	pub fn boundsFN(_: &Lua, id: usize) -> Result<(f32, f32, f32, f32), Error>
	{
		let anim = Window::getWorld().getCurrentEntity().getSprite(id);
		let size = anim.anim.getFrameSize();
		let p1 = anim.ts.getMatrix() * glam::vec4(0.0, 0.0, 0.0, 1.0);
		let p2 = anim.ts.getMatrix() * glam::vec4(size.x as f32, 0.0, 0.0, 1.0);
		let p3 = anim.ts.getMatrix() * glam::vec4(size.x as f32, size.y as f32, 0.0, 1.0);
		let p4 = anim.ts.getMatrix() * glam::vec4(0.0, size.y as f32, 0.0, 1.0);

		let min = p1.min(p2).min(p3).min(p4);
		let max = p1.max(p2).max(p3).max(p4);

		Ok((min.x, min.y, max.x - min.x, max.y - min.y))
	}

	pub fn setPosFN(_: &Lua, data: (usize, f32, f32)) -> Result<(), Error>
	{
		Window::getWorld().getCurrentEntity().getSprite(data.0).getTransform()
			.setPosition(glam::vec2(data.1, data.2)); Ok(())
	}

	pub fn addPosFN(_: &Lua, data: (usize, f32, f32)) -> Result<(), Error>
	{
		Window::getWorld().getCurrentEntity().getSprite(data.0).getTransform()
			.translate(glam::vec2(data.1, data.2)); Ok(())
	}

	pub fn getPosFN(_: &Lua, id: usize) -> Result<(f64, f64), Error>
	{
		let pos = Window::getWorld().getCurrentEntity().getSprite(id).getTransform().getPosition();
		Ok((pos.x as f64, pos.y as f64))
	}

	pub fn setRotationFN(_: &Lua, data: (usize, f32)) -> Result<(), Error>
	{
		Window::getWorld().getCurrentEntity().getSprite(data.0).getTransform().setRotation(data.1); Ok(())
	}

	pub fn addRotationFN(_: &Lua, data: (usize, f32)) -> Result<(), Error>
	{
		Window::getWorld().getCurrentEntity().getSprite(data.0).getTransform().rotate(data.1); Ok(())
	}

	pub fn getRotationFN(_: &Lua, id: usize) -> Result<f64, Error>
	{
		Ok(Window::getWorld().getCurrentEntity().getSprite(id).getTransform().getRotation() as f64)
	}

	pub fn setScaleFN(_: &Lua, data: (usize, f32, f32)) -> Result<(), Error>
	{
		Window::getWorld().getCurrentEntity().getSprite(data.0).getTransform()
			.setScale(glam::vec2(data.1, data.2)); Ok(())
	}

	pub fn addScaleFN(_: &Lua, data: (usize, f32, f32)) -> Result<(), Error>
	{
		Window::getWorld().getCurrentEntity().getSprite(data.0).getTransform()
			.scale(glam::vec2(data.1, data.2)); Ok(())
	}

	pub fn getScaleFN(_: &Lua, id: usize) -> Result<(f64, f64), Error>
	{
		let scale = Window::getWorld().getCurrentEntity().getSprite(id).getTransform().getScale();
		Ok((scale.x as f64, scale.y as f64))
	}

	pub fn setOriginFN(_: &Lua, data: (usize, f32, f32)) -> Result<(), Error>
	{
		Window::getWorld().getCurrentEntity().getSprite(data.0).getTransform()
			.setOrigin(glam::vec2(data.1, data.2)); Ok(())
	}

	pub fn getOriginFN(_: &Lua, id: usize) -> Result<(f64, f64), Error>
	{
		let origin = Window::getWorld().getCurrentEntity().getSprite(id).getTransform().getOrigin();
		Ok((origin.x as f64, origin.y as f64))
	}
}

impl Drawable for AnimatedSprite
{
	fn draw(&mut self)
	{
		let shader = Window::getCamera().getImgShader();
		self.update();
		shader.activate();
		shader.setInt("tex", 0);
		shader.setMat4("model", &self.ts.getMatrix().to_cols_array());
		shader.setVec4("color", self.color.to_array());
		unsafe
		{
			gl::BindVertexArray(self.vao);
			gl::ActiveTexture(gl::TEXTURE0);
			self.anim.bindTexture();
			gl::DrawArrays(gl::QUADS, 0, 4);
			gl::BindVertexArray(0);
			gl::BindBuffer(gl::ARRAY_BUFFER, 0);
		}
		shader.setInt("layer", shader.getInt("layer") + 1);
	}
}