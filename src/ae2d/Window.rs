use std::time::Instant;
use mlua::{Error, Integer, Lua, Number};
use sdl2::keyboard::Scancode;

use super::{envell::World::World, Camera::Camera, Network::Network, Programmable::{Programmable, Variable}, UI::UI};

#[derive(Clone, Copy, Debug)]
pub enum KeyAction
{
	Pressed = 0,
	Released = 1,
	PressedRepeat = 2,
	ReleasedRepeat = 3
}

impl PartialEq for KeyAction
{
	fn eq(&self, other: &Self) -> bool
	{
		*self as i32 == *other as i32
	}
}

#[derive(Clone, Copy, Debug)]
pub struct KeyEvent
{
	pub key: sdl2::keyboard::Scancode,
	pub mods: sdl2::keyboard::Mod,
	pub action: KeyAction
}

#[derive(Clone, Copy)]
pub struct MouseEvent
{
	pub btn: sdl2::mouse::MouseButton,
	pub clicks: u8,
	pub pos: glam::Vec2
}

pub struct Color
{
	name: String,
	value: sdl2::pixels::Color
}

pub struct Window
{
	video: sdl2::VideoSubsystem,
	window: Option<sdl2::video::Window>,
	events: sdl2::EventPump,
	running: bool,
	clearColor: sdl2::pixels::Color,
	keyEvent: Option<KeyEvent>,
	mouseEvent: Option<MouseEvent>,
	palette: Vec<Color>,
	deltaTime: f32,
	lastTime: std::time::Instant,
	mouse: sdl2::mouse::MouseUtil,
	lockCursor: bool,
	mouseDelta: glam::Vec2,
	ui: UI,
	vars: Programmable,
	minDeltaTime: f32,
	maxDeltaTime: f32,
	textInput: String,
	net: super::Network::Network,
	gl: Option<sdl2::video::GLContext>,
	cam: Camera,
	world: World
}

impl Window
{
	pub fn default() -> Window
	{
		let c = sdl2::init().expect("Failed to initialize SDL");
		Window
		{
			video: c.video().unwrap(),
			window: None,
			events: c.event_pump().unwrap(),
			running: true,
			clearColor: sdl2::pixels::Color::BLACK,
			deltaTime: 0.0,
			lastTime: Instant::now(),
			keyEvent: None,
			mouseEvent: None,
			palette: Vec::new(),
			mouse: c.mouse(),
			lockCursor: false,
			mouseDelta: glam::Vec2::ZERO,
			ui: UI::new(),
			vars: std::collections::HashMap::new(),
			minDeltaTime: 0.001,
			maxDeltaTime: 1.0,
			textInput: String::new(),
			net: super::Network::Network::new(),
			gl: None,
			cam: Camera::new(),
			world: World::new()
		}
	}

	fn getInstance() -> &'static mut Window
	{
		static mut INSTANCE: Option<Window> = None;
		
		unsafe
		{
			if INSTANCE.is_none() { INSTANCE = Some(Window::default()); }
			INSTANCE.as_mut().expect("Window singleton is not initialized")
		}
	}
	
	pub fn init()
	{
		let src = super::Assets::readXML("res/global/config.xml".to_string());
		if src.is_none() { return }
		let f = src.unwrap();

		let title = f.att_opt("title").unwrap_or("Ae2D").to_string();
		let size = glam::vec2(
			f.att_opt("w").unwrap_or("1280").parse().unwrap(),
			f.att_opt("h").unwrap_or("720").parse().unwrap()
		);
		let style = f.att_opt("style").unwrap_or("default").to_string();

		let mut pos = glam::Vec2::splat(-127.0);
		let mut hideCursor = false;
		let mut lockCursor = false;
		let mut vsync = true;
		let mut minDeltaTime = 0.001;
		let mut maxDeltaTime = 1.0;
		let mut clearColor = sdl2::pixels::Color::BLACK;
		
		let i = Window::getInstance();

		for x in f.elements()
		{
			let name = x.name().local_part();
			if name == "position"
			{
				let value: Vec<&str> = x.text().unwrap_or("-127 -127").split(" ").collect();
				pos = glam::vec2(
					value.get(0).unwrap().parse().unwrap(),
					value.get(1).unwrap().parse().unwrap()
				);
			}
			else if name == "hideCursor" { hideCursor = x.text().unwrap_or("false").parse().unwrap(); }
			else if name == "lockCursor" { lockCursor = x.text().unwrap_or("false").parse().unwrap(); }
			else if name == "vsync" { vsync = x.text().unwrap_or("false").parse().unwrap(); }
			else if name == "minDeltaTime" { minDeltaTime = x.text().unwrap_or("0.001").parse().unwrap(); }
			else if name == "maxDeltaTime" { maxDeltaTime = x.text().unwrap_or("1.0").parse().unwrap(); }
			else if name == "clearColor" { clearColor = Window::getColor(x.text().unwrap_or("black:").to_string()); }
			else if name == "color"
			{
				let value: Vec<&str> = x.text().unwrap_or("0 0 0 0").split(" ").collect();
				i.palette.push(Color {
					name: x.att_opt("name").unwrap_or("").to_string(),
					value: sdl2::pixels::Color::RGBA(
						value.get(0).unwrap().parse().unwrap(),
						value.get(1).unwrap().parse().unwrap(),
						value.get(2).unwrap().parse().unwrap(),
						value.get(3).unwrap().parse().unwrap()
					)
				});
			}
			else
			{
				let text = x.text().unwrap_or("").to_string();
				let var = text.parse::<f32>().unwrap_or(0.0);

				i.vars.insert(
					name.to_string(),
					super::Programmable::Variable {
						num: var,
						string: text
					}
				);
			}
		}

		i.minDeltaTime = minDeltaTime;
		i.maxDeltaTime = maxDeltaTime;
		i.clearColor = clearColor;

		let attr = i.video.gl_attr();
		attr.set_context_profile(sdl2::video::GLProfile::Core);
		attr.set_context_version(2, 1);
		attr.set_depth_size(24);

		let mut builder = i.video.window(title.as_str(), size.x as u32, size.y as u32);

		if pos != glam::Vec2::splat(-127.0) { builder.position(pos.x as i32, pos.y as i32); }
		else { builder.position_centered(); }
		if style.as_str() == "resizable" { builder.resizable(); }
		if style.as_str() == "borderless" { builder.borderless(); }
		if style.as_str() == "fullscreen" { builder.fullscreen_desktop(); }

		i.window = Some(builder.opengl().build().unwrap());

		i.gl = Some(i.window.as_mut().unwrap().gl_create_context().unwrap());
		gl::load_with(|name| i.video.gl_get_proc_address(name) as *const _);
		
		i.video.gl_set_swap_interval(if vsync { sdl2::video::SwapInterval::VSync } else { sdl2::video::SwapInterval::Immediate });
		i.mouse.show_cursor(!hideCursor);
		i.lockCursor = lockCursor;

		unsafe
		{
			gl::Enable(gl::DEPTH_TEST);
			gl::Enable(gl::BLEND);
			gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
			gl::DepthFunc(gl::LESS);
			let size = i.window.as_mut().unwrap().size();
			gl::Viewport(0, 0, size.0 as i32, size.1 as i32);
		}
		
		i.ui.load("res/ui/mainMenu.xml".to_string());
		i.cam.load();
	}

	pub fn update()
	{
		let i = Window::getInstance();
		i.keyEvent = None;
		i.mouseEvent = None;
		i.mouseDelta = glam::Vec2::ZERO;
		i.textInput.clear();

		i.deltaTime = f32::clamp(i.lastTime.elapsed().as_secs_f32(), i.minDeltaTime, i.maxDeltaTime);
		i.lastTime = std::time::Instant::now();

		for event in i.events.poll_iter()
		{
			match event
			{
				sdl2::event::Event::Quit {..} => { i.running = false; }
				sdl2::event::Event::KeyDown { scancode, keymod, repeat, .. } =>
				{
					i.keyEvent = Some(KeyEvent
					{
						key: scancode.unwrap(),
						mods: keymod,
						action: if repeat { KeyAction::PressedRepeat } else { KeyAction::Pressed }
					});
				},
				sdl2::event::Event::KeyUp { scancode, keymod, repeat, .. } =>
				{
					i.keyEvent = Some(KeyEvent
					{
						key: scancode.unwrap(),
						mods: keymod,
						action: if repeat { KeyAction::ReleasedRepeat } else { KeyAction::Released }
					});
				},
				sdl2::event::Event::MouseButtonDown { mouse_btn, clicks, x, y, .. } =>
				{
					i.mouseEvent = Some(MouseEvent
					{
						btn: mouse_btn,
						clicks,
						pos: glam::vec2(x as f32, y as f32)
					});
				},
				sdl2::event::Event::MouseButtonUp { mouse_btn, x, y, .. } =>
				{
					i.mouseEvent = Some(MouseEvent
					{
						btn: mouse_btn,
						clicks: 0,
						pos: glam::vec2(x as f32, y as f32)
					});
				},
				sdl2::event::Event::Window { win_event, .. } =>
				{
					match win_event
					{
						sdl2::event::WindowEvent::Resized(x, y) =>
						{
							unsafe { gl::Viewport(0, 0, x, y); }
						},
						sdl2::event::WindowEvent::Maximized =>
						{
							unsafe { gl::Viewport(0, 0, Window::getSize().x as i32, Window::getSize().y as i32); }
						},
						_ => {}
					}
					i.ui.resize();
					i.cam.updateWinProj();
				},
				sdl2::event::Event::MouseMotion { x, y, xrel, yrel, .. } =>
				{
					if x == Window::getSize().x as i32 / 2 && y == Window::getSize().y as i32 / 2 { continue; }
					i.mouseDelta = glam::vec2(xrel as f32, yrel as f32);
				},
				sdl2::event::Event::TextInput { text, .. } =>
				{
					i.textInput = text;
				},
				_ => {}
			}
		}

		if i.lockCursor { i.mouse.warp_mouse_in_window(
			i.window.as_ref().unwrap(),
			Window::getSize().x as i32 / 2,
			Window::getSize().y as i32 / 2
		); }

		i.world.update();
	}

	pub fn clear()
	{
		let i = Window::getInstance();
		
        unsafe
        {
            let c = Window::toGLcolor(i.clearColor);
            gl::ClearColor(c.0, c.1, c.2, c.3);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

	pub fn setSize(size: glam::Vec2)
	{
		Window::getInstance().window.as_mut().unwrap().set_size(
			size.x as u32,
			size.y as u32
		);
	}

	pub fn getSize() -> glam::Vec2
	{
		let size = Window::getInstance().window.as_mut().unwrap().size();
		glam::vec2(size.0 as f32, size.1 as f32)
	}

	pub fn isKeyPressed(key: sdl2::keyboard::Scancode) -> bool
	{
		Window::getInstance().events.keyboard_state().is_scancode_pressed(key)
	}

	pub fn isMousePressed(btn: sdl2::mouse::MouseButton) -> bool
	{
		Window::getInstance().events.mouse_state().is_mouse_button_pressed(btn)
	}

	pub fn getColor(name: String) -> sdl2::pixels::Color
	{
		for c in Window::getInstance().palette.iter()
		{
			if c.name == name { return c.value }
		}
		sdl2::pixels::Color::RGBA(0, 0, 0,0)
	}

    pub fn toGLcolor(clr: sdl2::pixels::Color) -> (f32, f32, f32, f32)
    {
        (
            clr.r as f32 / 255.0,
            clr.g as f32 / 255.0,
            clr.b as f32 / 255.0,
            clr.a as f32 / 255.0
        )
    }

    pub fn display()
    {
		Window::clear();
		
        let i = Window::getInstance();
		i.cam.update();
		i.world.render();
		i.ui.render();
		
        i.window.as_mut().unwrap().gl_swap_window();
    }

	pub fn getKeyEvent() -> Option<KeyEvent> { Window::getInstance().keyEvent }
	pub fn getMouseEvent() -> Option<MouseEvent> { Window::getInstance().mouseEvent }
	pub fn isOpen() -> bool { Window::getInstance().running }
	pub fn close() { Window::getInstance().running = false; }
	pub fn getDeltaTime() -> f32 { Window::getInstance().deltaTime }
	pub fn getMouseDelta() -> glam::Vec2 { Window::getInstance().mouseDelta }
	pub fn getUI() -> &'static mut UI { &mut Window::getInstance().ui }

	pub fn getGL() -> String
	{
		unsafe
		{
			let v = gl::GetString(gl::VERSION);
			let mut size: isize = 0;
			let mut vector: Vec<u8> = vec![];
			while v.offset(size).read() != 0
			{
				vector.push(v.offset(size).read());
				size += 1;
			}
			String::from_utf8(vector).unwrap()
		}
	}

	pub fn getCamera() -> &'static mut Camera { &mut Window::getInstance().cam }
	pub fn getWorld() -> &'static mut World { &mut Window::getInstance().world }
	pub fn getNetwork() -> &'static mut Network { &mut Window::getInstance().net }

	pub fn getMousePos() -> glam::IVec2
	{
		let s = Window::getInstance().events.mouse_state();
		glam::ivec2(s.x(), s.y())
	}

	pub fn getVariable(name: String) -> super::Programmable::Variable
	{
		Window::getInstance().vars
			.get(&name)
			.unwrap_or(
				&super::Programmable::Variable { num: 0.0, string: String::new() }
			).clone()
	}

	pub fn resetDT()
	{
		Window::getInstance().lastTime = std::time::Instant::now();
	}

	pub fn setLockCursor(lock: bool)
	{
		Window::getInstance().lockCursor = lock;
	}

	fn sizeFN(_: &Lua, _: ()) -> Result<(Integer, Integer), Error>
	{
		Ok((Window::getSize().x as i64, Window::getSize().y as i64))
	}

	fn dtFN(_: &Lua, _: ()) -> Result<Number, Error> { Ok(Window::getDeltaTime() as f64) }

	fn getNumFN(_: &Lua, name: String) -> Result<Number, Error>
	{
		Ok(Window::getVariable(name).num as f64)
	}

	fn getStrFN(_: &Lua, name: String) -> Result<String, Error>
	{
		Ok(Window::getVariable(name).string)
	}

	fn setNumFN(_: &Lua, options: (String, f32)) -> Result<(), Error>
	{
		Window::getInstance().vars.insert(
			options.0,
			Variable { num: options.1, string: String::new() }
		);
		Ok(())
	}

	fn setStrFN(_: &Lua, options: (String, String)) -> Result<(), Error>
	{
		Window::getInstance().vars.insert(
			options.0,
			Variable { num: 0.0, string: options.1 }
		);
		Ok(())
	}

	fn mousePosFN(_: &Lua, _: ()) -> Result<(Integer, Integer), Error>
	{
		Ok((
			Window::getMousePos().x as i64,
			Window::getMousePos().y as i64
		))
	}

	fn mousePressedFN(_: &Lua, name: String) -> Result<bool, Error>
	{
		let btn = match name.as_str()
		{
			"Left" => sdl2::mouse::MouseButton::Left,
			"Right" => sdl2::mouse::MouseButton::Right,
			"Middle" => sdl2::mouse::MouseButton::Middle,
			"X1" => sdl2::mouse::MouseButton::X1,
			"X2" => sdl2::mouse::MouseButton::X2,
			_ => sdl2::mouse::MouseButton::Unknown
		};
		Ok(Window::isMousePressed(btn))
	}

	fn mouseJustPressedFN(_: &Lua, name: String) -> Result<bool, Error>
	{
		let e = Window::getMouseEvent();
		if e.is_none() { return Ok(false); }
		let btn = match name.as_str()
		{
			"Left" => sdl2::mouse::MouseButton::Left,
			"Right" => sdl2::mouse::MouseButton::Right,
			"Middle" => sdl2::mouse::MouseButton::Middle,
			"X1" => sdl2::mouse::MouseButton::X1,
			"X2" => sdl2::mouse::MouseButton::X2,
			_ => sdl2::mouse::MouseButton::Unknown
		};
		Ok(e.unwrap().btn == btn && e.unwrap().clicks > 0)
	}

	fn keyPressedFN(_: &Lua, name: String) -> Result<bool, Error>
	{
		let scancode = Scancode::from_name(&name).unwrap_or(Scancode::SysReq);
		Ok(Window::isKeyPressed(scancode))
	}

	fn keyJustPressedFN(_: &Lua, name: String) -> Result<bool, Error>
	{
		let e = Window::getKeyEvent();
		if e.is_none() { return Ok(false); }
		let scancode = Scancode::from_name(&name).unwrap_or(Scancode::SysReq);
		Ok(
			e.unwrap().key == scancode &&
			(
				e.unwrap().action == KeyAction::Pressed ||
				e.unwrap().action == KeyAction::PressedRepeat
			)
		)
	}

	fn keyModPressedFN(_: &Lua, name: String) -> Result<bool, Error>
	{
		let e = Window::getKeyEvent();
		if e.is_none() { return Ok(false); }
		Ok(
			(name == "LControl" && e.unwrap().mods.intersects(sdl2::keyboard::Mod::LCTRLMOD)) ||
			(name == "RControl" && e.unwrap().mods.intersects(sdl2::keyboard::Mod::RCTRLMOD)) ||
			(name == "LShift" && e.unwrap().mods.intersects(sdl2::keyboard::Mod::LSHIFTMOD)) ||
			(name == "RShift" && e.unwrap().mods.intersects(sdl2::keyboard::Mod::RSHIFTMOD)) ||
			(name == "LAlt" && e.unwrap().mods.intersects(sdl2::keyboard::Mod::LALTMOD)) ||
			(name == "RAlt" && e.unwrap().mods.intersects(sdl2::keyboard::Mod::RALTMOD))
		)
	}

	fn closeFN(_: &Lua, _: ()) -> Result<(), Error> { Window::close(); Ok(()) }

	fn executeFN(state: &Lua, code: String) -> Result<(), Error>
	{
		state.load(code).exec();
		Ok(())
	}

	fn inputFN(_: &Lua, _: ()) -> Result<String, Error> { Ok(Window::getInstance().textInput.clone()) }

	fn getClipboardFN(_: &Lua, _: ()) -> Result<String, Error>
	{
		Ok(Window::getInstance().video.clipboard().clipboard_text().unwrap_or(String::new()))
	}

	fn uiLoadFN(_: &Lua, path: String) -> Result<(), Error>
	{
		Window::getUI().requestReload(path);
		Ok(())
	}

	fn worldLoadFN(_: &Lua, path: String) -> Result<(), Error>
	{
		Window::getInstance().world.load(path);
		Ok(())
	}

	fn camSizeFN(_: &Lua, _: ()) -> Result<(Number, Number), Error>
	{
		Ok((
			Window::getCamera().getSize().x as f64,
			Window::getCamera().getSize().y as f64
		))
	}

	pub fn initLua(script: &Lua)
	{
		let table = script.create_table().unwrap();

		table.set("size", script.create_function(Window::sizeFN).unwrap());
		table.set("dt", script.create_function(Window::dtFN).unwrap());
		table.set("getNum", script.create_function(Window::getNumFN).unwrap());
		table.set("getStr", script.create_function(Window::getStrFN).unwrap());
		table.set("setNum", script.create_function(Window::setNumFN).unwrap());
		table.set("setStr", script.create_function(Window::setStrFN).unwrap());
		table.set("mousePos", script.create_function(Window::mousePosFN).unwrap());
		table.set("mousePressed", script.create_function(Window::mousePressedFN).unwrap());
		table.set("mouseJustPressed", script.create_function(Window::mouseJustPressedFN).unwrap());
		table.set("keyModPressed", script.create_function(Window::keyModPressedFN).unwrap());
		table.set("close", script.create_function(Window::closeFN).unwrap());
		table.set("keyPressed", script.create_function(Window::keyPressedFN).unwrap());
		table.set("keyJustPressed", script.create_function(Window::keyJustPressedFN).unwrap());
		table.set("execute", script.create_function(Window::executeFN).unwrap());
		table.set("input", script.create_function(Window::inputFN).unwrap());
		table.set("getClipboard", script.create_function(Window::getClipboardFN).unwrap());
		table.set("loadUI", script.create_function(Window::uiLoadFN).unwrap());
		table.set("loadWorld", script.create_function(Window::worldLoadFN).unwrap());
		table.set("camSize", script.create_function(Window::camSizeFN).unwrap());

		script.globals().set("window", table);

		Network::initLua(script);
	}
}
