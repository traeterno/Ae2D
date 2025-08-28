use std::collections::HashMap;
use glfw::Context;

use crate::ae2d::{Network::Network, Shader::Shader, World::World};

use super::{Camera::Camera, Programmable::{Programmable, Variable}, UI::UI};

pub struct Window
{
	context: glfw::Glfw,
	pub window: Option<glfw::PWindow>,
	events: Option<glfw::GlfwReceiver<(f64, glfw::WindowEvent)>>,
	deltaTime: f32,
	lastTime: std::time::Instant,
	pub prog: Programmable,
	pub mouseEvent: Option<(glfw::MouseButton, glfw::Action, glfw::Modifiers)>,
	pub keyEvent: Option<(glfw::Key, glfw::Action, glfw::Modifiers)>,
	pub inputEvent: Option<char>,
	cam: Camera,
	textures: HashMap<String, u32>,
	ui: UI,
	net: Network,
	world: World,
	shaders: HashMap<String, Shader>,
	server: Option<std::process::Child>
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
			deltaTime: 0.0,
			lastTime: std::time::Instant::now(),
			prog: Programmable::new(),
			mouseEvent: None,
			keyEvent: None,
			cam: Camera::new(),
			textures: HashMap::new(),
			ui: UI::new(),
			net: Network::new(),
			inputEvent: None,
			world: World::new(),
			shaders: HashMap::new(),
			server: None
		}
	}

	pub fn getInstance() -> &'static mut Window
	{
		static mut INSTANCE: Option<Window> = None;
		unsafe
		{
			if INSTANCE.is_none() { INSTANCE = Some(Window::default()); }
			INSTANCE.as_mut().unwrap()
		}
	}
	
	pub fn init(path: &str)
	{
		let cfg = json::parse(
			&std::fs::read_to_string(path)
			.unwrap_or(String::new())
		);
		if cfg.is_err() { return; }
		let cfg = cfg.unwrap();
		
		let i = Window::getInstance();

		let mut title = "Ae2D";
		let mut size = glam::vec2(1280.0, 720.0);
		let mut vsync = true;
		let mut fullscreen = false;
		let mut colors = HashMap::new();
		let mut uiPath = "";

		for (name, section) in cfg.entries()
		{
			if name == "main"
			{
				for (x, y) in section.entries()
				{
					if x == "title" { title = y.as_str().unwrap(); }
					if x == "vsync" { vsync = y.as_bool().unwrap(); }
					if x == "fullscreen" { fullscreen = y.as_bool().unwrap(); }
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
					if x == "uiPath"
					{
						uiPath = y.as_str().unwrap();
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
					let s = value.as_str().unwrap_or_default().to_string();
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

		let (mut window, events) = if fullscreen
		{
			vsync = true;
			i.context.with_primary_monitor(|g, m|
			{
				let s = m.as_ref().unwrap().get_video_mode().unwrap();
				size = glam::vec2(s.width as f32, s.height as f32);
				g.create_window(
					size.x as u32,
					size.y as u32,
					title,
					glfw::WindowMode::FullScreen(m.unwrap())
				).unwrap()
			})
		}
		else
		{
			i.context.create_window(
				size.x as u32,
				size.y as u32,
				title,
				glfw::WindowMode::Windowed
			).unwrap()
		};

		window.set_mouse_button_polling(true);
		window.set_key_polling(true);
		window.set_size_polling(true);
		window.set_char_polling(true);
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
			gl::StencilMask(0xFF);
			gl::StencilFunc(gl::NOTEQUAL, 1, 0xFF);
			gl::StencilOp(gl::KEEP, gl::REPLACE, gl::REPLACE);
			gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
			gl::Viewport(0, 0, size.x as i32, size.y as i32);
		}

		i.ui.load(uiPath);
	}

	pub fn update()
	{
		let i = Window::getInstance();

		i.mouseEvent = None;
		i.keyEvent = None;
		i.inputEvent = None;
		i.deltaTime = i.lastTime.elapsed().as_secs_f32().min(0.1);
		i.lastTime = std::time::Instant::now();
		
		let events = i.events.as_ref().unwrap();
		let window = i.window.as_mut().unwrap();
		
		i.context.poll_events();
		for (_, event) in glfw::flush_messages(events)
		{
			match event
			{
				glfw::WindowEvent::Close =>
				{
					window.set_should_close(true);
					i.net = Network::new();
				}
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
					i.cam.setSize(false, (w, h));
					i.ui.resize();
					unsafe
					{
						gl::Viewport(0, 0, w, h);
					}
				},
				glfw::WindowEvent::Char(c) =>
				{
					i.inputEvent = Some(c);
				}
				e => println!("{e:?}")
			}
		}

		i.world.update();
		i.ui.update();
	}

	pub fn render()
	{
		let i = Window::getInstance();

		i.cam.clear();
		i.cam.toggleTransform(true);
		i.cam.draw(&mut i.world);
		i.cam.toggleTransform(false);
		i.cam.display();
		i.cam.draw(&mut i.ui);
		i.window.as_mut().unwrap().swap_buffers();
	}

	pub fn display()
	{
		Window::getInstance().window.as_mut().unwrap().swap_buffers();
	}

	pub fn getSize() -> (i32, i32)
	{
		Window::getInstance().window.as_ref().unwrap().get_size()
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

	pub fn strToMB(name: String) -> glfw::MouseButton
	{
		match name.as_str()
		{
			"Left" => glfw::MouseButton::Button1,
			"Right" => glfw::MouseButton::Button2,
			"Middle" => glfw::MouseButton::Button3,
			_ => glfw::MouseButton::Button8
		}
	}

	pub fn strToKey(name: String) -> glfw::Key
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
			"Escape" => glfw::Key::Escape,
			"Enter" => glfw::Key::Enter,
			"Backspace" => glfw::Key::Backspace,
			"Space" => glfw::Key::Space,
			"F1" => glfw::Key::F1,
			"F2" => glfw::Key::F2,
			"F3" => glfw::Key::F3,
			"F4" => glfw::Key::F4,
			"F5" => glfw::Key::F5,
			"F6" => glfw::Key::F6,
			"F7" => glfw::Key::F7,
			"F8" => glfw::Key::F8,
			"F9" => glfw::Key::F9,
			"F10" => glfw::Key::F10,
			"F11" => glfw::Key::F11,
			"F12" => glfw::Key::F12,
			"Left" => glfw::Key::Left,
			"Right" => glfw::Key::Right,
			"Up" => glfw::Key::Up,
			"Down" => glfw::Key::Down,
			"Home" => glfw::Key::Home,
			"End" => glfw::Key::End,
			"LShift" => glfw::Key::LeftShift,
			"RShift" => glfw::Key::RightShift,
			"LCtrl" => glfw::Key::LeftControl,
			"RCtrl" => glfw::Key::RightControl,
			"LAlt" => glfw::Key::LeftAlt,
			"RAlt" => glfw::Key::RightAlt,
			_ => glfw::Key::Unknown
		}
	}

	pub fn strToMod(name: String) -> glfw::Modifiers
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

	pub fn getShader(name: String) -> &'static Shader
	{
		if let Some(s) = Window::getInstance().shaders.get(&name)
		{
			return s;
		}

		Window::getInstance().shaders.insert(name.clone(), Shader::load(
			&(String::from("res/shaders/") + &name + ".vert"),
			&(String::from("res/shaders/") + &name + ".frag")
		));

		Window::getInstance().shaders.get(&name).unwrap()
	}

	pub fn getNetwork() -> &'static mut Network
	{
		&mut Window::getInstance().net
	}

	pub fn getWorld() -> &'static mut World
	{
		&mut Window::getInstance().world
	}

	pub fn clearCache()
	{
		let i = Window::getInstance();
		i.textures.clear();
		i.shaders.clear();
	}

	pub fn updateMatrices(proj: glam::Mat4, view: glam::Mat4)
	{
		for (_, s) in &mut Window::getInstance().shaders
		{
			s.activate();
			s.setMat4("projection", proj);
			s.setMat4("view", view);
		}
	}

	pub fn launchServer()
	{
		let i = Window::getInstance();
		let path = if cfg!(debug_assertions)
		{
			"./target/debug/envell.exe"
		} else { "./res/system/server.exe" };
		i.server = Some(
			std::process::Command::new(path).arg("silent").spawn().unwrap()
		);
	}

	pub fn setMousePos(pos: glam::Vec2)
	{
		let w = Window::getInstance().window.as_mut().unwrap();
		w.set_cursor_pos(
			pos.x as f64,
			pos.y as f64
		);
	}
}