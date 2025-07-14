use std::collections::HashMap;
use glfw::Context;
use mlua::{Error, Lua};

use super::{Camera::Camera, Programmable::{Programmable, Variable}, UI::UI};

pub struct Window
{
	context: glfw::Glfw,
	window: Option<glfw::PWindow>,
	events: Option<glfw::GlfwReceiver<(f64, glfw::WindowEvent)>>,
	_palette: HashMap<String, glam::Vec4>,
	deltaTime: f32,
	lastTime: std::time::Instant,
	prog: Programmable,
	mouseEvent: Option<(glfw::MouseButton, glfw::Action, glfw::Modifiers)>,
	keyEvent: Option<(glfw::Key, glfw::Action, glfw::Modifiers)>,
	cam: Camera,
	textures: HashMap<String, u32>,
	ui: UI
}

impl Window
{
	pub fn default() -> Window
	{
		use glfw::fail_on_errors;
		Window
		{
			context: glfw::init(glfw::fail_on_errors!()).unwrap(),
			window: None,
			events: None,
			_palette: HashMap::new(),
			deltaTime: 0.0,
			lastTime: std::time::Instant::now(),
			prog: Programmable::new(),
			mouseEvent: None,
			keyEvent: None,
			cam: Camera::new(),
			textures: HashMap::new(),
			ui: UI::new()
		}
	}

	fn getInstance() -> &'static mut Window
	{
		static mut INSTANCE: Option<Window> = None;
		
		unsafe
		{
			if INSTANCE.is_none()
			{
				println!("Initializing Window");
				INSTANCE = Some(Window::default());
			}
			INSTANCE.as_mut().unwrap()
		}
	}
	
	pub fn init()
	{
		let cfg = json::parse(
			&std::fs::read_to_string("res/global/config.json")
			.unwrap_or(String::new())
		);
		if cfg.is_err() { return; }
		let cfg = cfg.unwrap();
		
		let i = Window::getInstance();

		let mut title = "Ae2D";
		let mut size = glam::vec2(1280.0, 720.0);
		let mut vsync = true;
		let mut colors = HashMap::new();

		for (name, section) in cfg.entries()
		{
			if name == "main"
			{
				for (x, y) in section.entries()
				{
					if x == "title" { title = y.as_str().unwrap(); }
					if x == "vsync" { vsync = y.as_bool().unwrap(); }
					if x == "size"
					{
						let mut s = y.members();
						size = glam::vec2(
							s.nth(0).unwrap().as_f32().unwrap(),
							s.nth(0).unwrap().as_f32().unwrap()
						);
					}
					if x == "uiSize"
					{
						let mut s = y.members();
						i.ui.setSize(glam::vec2(
							s.nth(0).unwrap().as_f32().unwrap(),
							s.nth(0).unwrap().as_f32().unwrap()
						));
					}
				}
			}
			if name == "colors"
			{
				for (clr, value) in section.entries()
				{
					let mut c = value.members();
					colors.insert(clr, glam::vec4(
						c.nth(0).unwrap().as_f32().unwrap(),
						c.nth(0).unwrap().as_f32().unwrap(),
						c.nth(0).unwrap().as_f32().unwrap(),
						c.nth(0).unwrap().as_f32().unwrap()
					));
				}
			}
			if name == "custom"
			{
				for (name, value) in section.entries()
				{
					let num = value.as_f32().unwrap_or(0.0);
					let s = value.as_str().unwrap().to_string();
					i.prog.insert(
						name.to_string(),
						Variable
						{
							num,
							string: if num == 0.0 { s } else { String::new()}
						}
					);
				}
			}
		}

		let (mut window, events) = i.context.create_window(
			size.x as u32,
			size.y as u32,
			title,
			glfw::WindowMode::Windowed
		).unwrap();

		window.set_mouse_button_polling(true);
		window.set_key_polling(true);
		window.set_size_polling(true);
		window.make_current();
		
		gl::load_with(|name| i.context.get_proc_address_raw(name));

		i.context.set_swap_interval(
			if vsync { glfw::SwapInterval::Sync(1) }
			else { glfw::SwapInterval::None }
		);

		i.window = Some(window);
		i.events = Some(events);

		i.cam.load();
		
		unsafe
		{
			gl::Enable(gl::BLEND);
			gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
			gl::DepthFunc(gl::LESS);
			gl::Viewport(0, 0, size.x as i32, size.y as i32);
		}

		i.ui.load(String::from("res/ui/mainMenu.json"));
	}

	pub fn update()
	{
		let i = Window::getInstance();

		i.mouseEvent = None;
		i.keyEvent = None;
		i.deltaTime = i.lastTime.elapsed().as_secs_f32();
		i.lastTime = std::time::Instant::now();
		
		let events = i.events.as_ref().unwrap();
		let window = i.window.as_mut().unwrap();
		
		i.context.poll_events();
		for (_, event) in glfw::flush_messages(events)
		{
			match event
			{
				glfw::WindowEvent::Close => window.set_should_close(true),
				glfw::WindowEvent::MouseButton(b, a, m) =>
				{
					i.mouseEvent = Some((b, a, m));
				}
				glfw::WindowEvent::Key(k, _, a, m) =>
				{
					i.keyEvent = Some((k, a, m));
				}
				glfw::WindowEvent::Size(w, h) =>
				{
					i.cam.setSize((w, h));
					unsafe
					{
						gl::Viewport(0, 0, w, h);
					}
				}
				e => println!("{e:?}")
			}
		}

		i.ui.update();

		i.cam.clear();
		i.cam.toggleTransform(false);
		i.cam.draw(&mut i.ui);
		i.cam.display();
		window.swap_buffers();
	}

	pub fn getSize() -> (i32, i32)
	{
		Window::getInstance().window.as_ref().unwrap().get_size()
	}

	pub fn _getColor(name: String) -> glam::Vec4
	{
		*Window::getInstance()._palette.get(&name).unwrap_or(
			&glam::Vec4::ZERO
		)
	}

	pub fn isOpen() -> bool
	{
		!Window::getInstance().window.as_ref().unwrap().should_close()
	}

	pub fn close()
	{
		Window::getInstance().window.as_mut().unwrap().set_should_close(true);
	}

	pub fn getCamera() -> &'static mut Camera
	{
		&mut Window::getInstance().cam
	}

	pub fn getUI() -> &'static mut UI
	{
		&mut Window::getInstance().ui
	}

	pub fn getDeltaTime() -> f32 { Window::getInstance().deltaTime }

	pub fn resetDT()
	{
		Window::getInstance().lastTime = std::time::Instant::now();
	}

	fn sizeFN(_: &Lua, _: ()) -> Result<(i32, i32), Error>
	{
		Ok(Window::getSize())
	}

	fn dtFN(_: &Lua, _: ()) -> Result<f32, Error> { Ok(Window::getDeltaTime()) }

	fn getNumFN(_: &Lua, name: String) -> Result<f32, Error>
	{
		Ok(Window::getInstance().prog.get(&name)
			.unwrap_or(&Variable::new()).num)
	}

	fn getStrFN(_: &Lua, name: String) -> Result<String, Error>
	{
		Ok(Window::getInstance().prog.get(&name)
			.unwrap_or(&Variable::new()).string.clone())
	}

	fn setNumFN(_: &Lua, options: (String, f32)) -> Result<(), Error>
	{
		Window::getInstance().prog.insert(
			options.0,
			Variable { num: options.1, string: String::new() }
		);
		Ok(())
	}

	fn setStrFN(_: &Lua, options: (String, String)) -> Result<(), Error>
	{
		Window::getInstance().prog.insert(
			options.0,
			Variable { num: 0.0, string: options.1 }
		);
		Ok(())
	}

	fn mousePosFN(_: &Lua, _: ()) -> Result<(i32, i32), Error>
	{
		let p = Window::getInstance().window.as_ref().unwrap()
			.get_cursor_pos();
		Ok((p.0 as i32, p.1 as i32))
	}

	fn mousePressedFN(_: &Lua, name: String) -> Result<bool, Error>
	{
		let btn = Window::strToMB(name);
		Ok(Window::getInstance().window.as_ref().unwrap()
			.get_mouse_button(btn) == glfw::Action::Press)
	}

	fn mouseJustPressedFN(_: &Lua, name: String) -> Result<bool, Error>
	{
		let e = Window::getInstance().mouseEvent;
		if e.is_none() { return Ok(false); }
		let e = e.unwrap();
		let btn = Window::strToMB(name);
		Ok(e.0 == btn && e.1 == glfw::Action::Press)
	}

	fn keyPressedFN(_: &Lua, name: String) -> Result<bool, Error>
	{
		Ok(Window::getInstance().window.as_ref().unwrap()
			.get_key(Window::strToKey(name)) == glfw::Action::Press)
	}

	fn keyJustPressedFN(_: &Lua, name: String) -> Result<bool, Error>
	{
		let e = Window::getInstance().keyEvent;
		if e.is_none() { return Ok(false); }
		let e = e.unwrap();
		Ok(e.0 == Window::strToKey(name) && e.1 == glfw::Action::Press)
	}

	fn keyModPressedFN(_: &Lua, name: String) -> Result<bool, Error>
	{
		let e = Window::getInstance().keyEvent;
		if e.is_none() { return Ok(false); }
		let e = e.unwrap();
		Ok(e.2.intersects(Window::strToMod(name)))
	}

	fn closeFN(_: &Lua, _: ()) -> Result<(), Error> { Window::close(); Ok(()) }

	fn executeFN(state: &Lua, code: String) -> Result<(), Error>
	{
		let _ = state.load(code).exec();
		Ok(())
	}

	pub fn initLua(script: &Lua)
	{
		let table = script.create_table().unwrap();

		let _ = table.set("size", script.create_function(Window::sizeFN).unwrap());
		let _ = table.set("dt", script.create_function(Window::dtFN).unwrap());
		let _ = table.set("getNum", script.create_function(Window::getNumFN).unwrap());
		let _ = table.set("getStr", script.create_function(Window::getStrFN).unwrap());
		let _ = table.set("setNum", script.create_function(Window::setNumFN).unwrap());
		let _ = table.set("setStr", script.create_function(Window::setStrFN).unwrap());
		let _ = table.set("mousePos", script.create_function(Window::mousePosFN).unwrap());
		let _ = table.set("mousePressed", script.create_function(Window::mousePressedFN).unwrap());
		let _ = table.set("mouseJustPressed", script.create_function(Window::mouseJustPressedFN).unwrap());
		let _ = table.set("keyModPressed", script.create_function(Window::keyModPressedFN).unwrap());
		let _ = table.set("close", script.create_function(Window::closeFN).unwrap());
		let _ = table.set("keyPressed", script.create_function(Window::keyPressedFN).unwrap());
		let _ = table.set("keyJustPressed", script.create_function(Window::keyJustPressedFN).unwrap());
		let _ = table.set("execute", script.create_function(Window::executeFN).unwrap());

		let _ = script.globals().set("window", table);

		// Network::initLua(script);
	}

	fn strToMB(name: String) -> glfw::MouseButton
	{
		match name.as_str()
		{
			"Left" => glfw::MouseButton::Button1,
			"Right" => glfw::MouseButton::Button2,
			"Middle" => glfw::MouseButton::Button3,
			_ => glfw::MouseButton::Button8
		}
	}

	fn strToKey(name: String) -> glfw::Key
	{
		match name.as_str()
		{
			"A" => glfw::Key::A,
			"B" => glfw::Key::B,
			"C" => glfw::Key::C,
			"D" => glfw::Key::D,
			"E" => glfw::Key::E,
			"F" => glfw::Key::F,
			"G" => glfw::Key::G,
			"H" => glfw::Key::H,
			"I" => glfw::Key::I,
			"J" => glfw::Key::J,
			"K" => glfw::Key::K,
			"L" => glfw::Key::L,
			"M" => glfw::Key::M,
			"N" => glfw::Key::N,
			"O" => glfw::Key::O,
			"P" => glfw::Key::P,
			"Q" => glfw::Key::Q,
			"R" => glfw::Key::R,
			"S" => glfw::Key::S,
			"T" => glfw::Key::T,
			"U" => glfw::Key::U,
			"V" => glfw::Key::V,
			"W" => glfw::Key::W,
			"X" => glfw::Key::X,
			"Y" => glfw::Key::Y,
			"Z" => glfw::Key::Z,
			"Num0" => glfw::Key::Num0,
			"Num1" => glfw::Key::Num1,
			"Num2" => glfw::Key::Num2,
			"Num3" => glfw::Key::Num3,
			"Num4" => glfw::Key::Num4,
			"Num5" => glfw::Key::Num5,
			"Num6" => glfw::Key::Num6,
			"Num7" => glfw::Key::Num7,
			"Num8" => glfw::Key::Num8,
			"Num9" => glfw::Key::Num9,
			_ => glfw::Key::Unknown
		}
	}

	fn strToMod(name: String) -> glfw::Modifiers
	{
		match name.as_str()
		{
			"Control" => glfw::Modifiers::Control,
			"Shift" => glfw::Modifiers::Shift,
			"Alt" => glfw::Modifiers::Alt,
			"Super" => glfw::Modifiers::Super,
			"NumLock" => glfw::Modifiers::NumLock,
			_ => glfw::Modifiers::CapsLock
		}
	}

	pub fn getTexture(path: String) -> u32
	{
		let tex = &mut Window::getInstance().textures;
		if let Some(t) = tex.get(&path) { return *t; }
		
		match stb_image::image::load(path.clone())
		{
			stb_image::image::LoadResult::ImageU8(data) =>
			{
				let mut t = 0;
				unsafe
				{
					gl::GenTextures(1, &mut t);
					gl::BindTexture(gl::TEXTURE_2D, t);

					gl::TexImage2D(
						gl::TEXTURE_2D,
						0,
						gl::RGBA as i32,
						data.width as i32,
						data.height as i32,
						0,
						gl::RGBA,
						gl::UNSIGNED_BYTE,
						data.data.as_ptr() as *const _
					);
					
					gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
					gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
					gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
					gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
				}

				tex.insert(path, t);
				t
			},
			_ => 0
		}
	}
}