use super::{Shader::Shader, Transformable::Transformable2D, Window::Window};

pub trait Drawable
{
	fn draw(&mut self);
}

/*
	TODO
	Implement 'drawable' trait on all the UI and sprites;
	after that add the Camera::draw() function to make layer constant
*/
pub struct Camera
{
	pos: glam::Vec2,
	imgShader: Shader,
	txtShader: Shader,
	shapeShader: Shader,
	projection: [f32; 16],
	winProj: [f32; 16],
	ts: Transformable2D,
	size: glam::Vec2
}

impl Camera
{
	pub fn new() -> Self
	{
		Self
		{
			pos: glam::Vec2::ZERO,
			imgShader: Shader::new(),
			txtShader: Shader::new(),
			shapeShader: Shader::new(),
			projection: glam::Mat4::IDENTITY.to_cols_array(),
			winProj: glam::Mat4::IDENTITY.to_cols_array(),
			ts: Transformable2D::new(),
			size: glam::Vec2::ZERO
		}
	}

	pub fn load(&mut self)
	{
		self.imgShader.load("res/shaders/image.vert", "res/shaders/image.frag");
		self.txtShader.load("res/shaders/text.vert", "res/shaders/text.frag");
		self.shapeShader.load("res/shaders/shape.vert", "res/shaders/shape.frag");
		self.updateWinProj();
	}

	pub fn updateWinProj(&mut self)
	{
		let s = Window::getSize();
		self.winProj = glam::Mat4::orthographic_rh_gl(
			0.0, s.x,
			s.y, 0.0,
			-100.0, 100.0
		).to_cols_array();
	}

	pub fn update(&mut self)
	{
		self.imgShader.activate();
		self.imgShader.setInt("layer", -99);

		self.txtShader.activate();
		self.txtShader.setInt("layer", -99);

		self.shapeShader.activate();
		self.shapeShader.setInt("layer", -99);
	}

	pub fn toggleCameraTransform(&mut self, enable: bool)
	{
		self.imgShader.activate();
		self.imgShader.setMat4("projection", &(if enable {self.projection} else {self.winProj}));
		self.imgShader.setMat4("view", &(if enable {self.ts.getMatrix()} else {glam::Mat4::IDENTITY}).to_cols_array());

		self.txtShader.activate();
		self.txtShader.setMat4("projection", &(if enable {self.projection} else {self.winProj}));
		self.txtShader.setMat4("view", &(if enable {self.ts.getMatrix()} else {glam::Mat4::IDENTITY}).to_cols_array());

		self.shapeShader.activate();
		self.shapeShader.setMat4("projection", &(if enable {self.projection} else {self.winProj}));
		self.shapeShader.setMat4("view", &(if enable {self.ts.getMatrix()} else {glam::Mat4::IDENTITY}).to_cols_array());
	}

	pub fn setSize(&mut self, s: glam::Vec2)
	{
		self.size = s;
		self.projection = glam::Mat4::orthographic_rh_gl(
			0.0, s.x,
			s.y, 0.0,
			-100.0, 100.0
		).to_cols_array();
	}

	pub fn getSize(&mut self) -> glam::Vec2 { self.size }

	pub fn draw(&mut self, obj: &mut impl Drawable)
	{
		obj.draw();
		let imgLayer = self.imgShader.getInt("layer");
		let txtLayer = self.txtShader.getInt("layer");
		let shapeLayer = self.shapeShader.getInt("layer");
		self.imgShader.activate();
		self.imgShader.setInt("layer", imgLayer.max(txtLayer.max(shapeLayer)));
		self.txtShader.activate();
		self.txtShader.setInt("layer", imgLayer.max(txtLayer.max(shapeLayer)));
		self.shapeShader.activate();
		self.shapeShader.setInt("layer", imgLayer.max(txtLayer.max(shapeLayer)));
	}
	
	pub fn getTransform(&mut self) -> &mut Transformable2D { &mut self.ts }
	pub fn getImgShader(&mut self) -> &mut Shader { &mut self.imgShader }
	pub fn getTxtShader(&mut self) -> &mut Shader { &mut self.txtShader }
	pub fn getShapeShader(&mut self) -> &mut Shader { &mut self.shapeShader }
}