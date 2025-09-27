#![allow(non_snake_case, static_mut_refs, dead_code)]

use std::sync::LazyLock;

use crate::ae2d::{Shapes, Skeleton::Skeleton, Window::Window};

mod ae2d;
mod server;

#[derive(Clone, Copy)]
enum FileType { Rig, SpriteList, AnimList }

static mut PATH: LazyLock<(FileType, String)> = LazyLock::new(|| (FileType::Rig, String::new()));
static mut SKELETON: LazyLock<Skeleton> = LazyLock::new(|| Skeleton::new());
static mut FILESELECTION: LazyLock<FileType> = LazyLock::new(|| FileType::Rig);

fn initExec()
{
	let s = Window::getUI().getObject("toolbox".to_string()).getScript();
	let _ = s.globals().raw_set(
		"Exec",
		s.create_function(|_, cmd: String|
		{
			println!("Called \"{cmd}\"");
			exec(cmd); Ok(())
		}).unwrap()
	);
}

fn exec(cmd: String)
{
	let args = cmd.split(" ")
		.collect::<Vec<&str>>();
	if args[0] == "page"
	{
		unsafe
		{
			if args[1] == "0" { *FILESELECTION = FileType::Rig; }
			if args[1] == "1" { *FILESELECTION = FileType::SpriteList; }
			if args[1] == "2" { *FILESELECTION = FileType::AnimList; }
		}
	}
	if args[0] == "reload"
	{
		unsafe
		{
			let (kind, path) = &*PATH;
			match *kind
			{
				FileType::Rig => (*SKELETON).loadRig(path.clone()),
				FileType::SpriteList => (*SKELETON).loadSprites(path.clone()),
				_ => {}
			}
		}
	}
}

#[derive(PartialEq, Clone, Copy)]
enum Action
{
	MoveCam
}

fn main()
{
	Window::init("res/global/se.json");
	let cam = Window::getCamera();
	cam.setSize(true, Window::getSize());
	cam.getTransformable().setOrigin(glam::vec2(
		(-Window::getSize().0 as f32 / 4.0 * 3.0) * 0.5,
		-Window::getSize().1 as f32 * 0.5
	));

	initExec();

	unsafe { gl::Enable(gl::BLEND); }

	let mut mpos = (0.0, 0.0);
	let mut maction = None;
	let mut scale = 1.0f32;

	while Window::isOpen()
	{
		Window::update();

		let (mx, my) = Window::getInstance().window.as_mut()
			.unwrap().get_cursor_pos();
		let mx = mx as f32;
		let my = my as f32;
		let ws =
			mx <= Window::getSize().0 as f32 / 4.0 * 3.0 &&
			my <= Window::getSize().1 as f32 / 4.0 * 3.0;

		if let Some(e) = Window::getInstance().keyEvent
		{
			if e.0 == glfw::Key::F1 && e.1 == glfw::Action::Press
			{
				Window::getUI().load("res/ui/se.json");
				initExec();
			}
		}
		if let Some(e) = Window::getInstance().mouseEvent
		{
			let pressed = e.1 == glfw::Action::Press;
			if e.0 == glfw::MouseButtonMiddle
			{
				maction = if pressed && ws { Some(Action::MoveCam) } else { None };
			}
		}
		if let Some(e) = Window::getInstance().scrollEvent
		{
			if ws
			{
				let dist =
					if e < 0.0 { 0.5 * e.abs() }
					else { 2.0 * e };
				scale *= dist;
				let sx = Window::getSize().0 as f32 * scale;
				let sy = Window::getSize().1 as f32 * scale;
				cam.setSize(true, (
					sx as i32,
					sy as i32
				));
				cam.getTransformable().setOrigin(glam::vec2(
					-sx * 0.5,
					-sy * 0.5
				));
			}
		}
		if let Some(file) = &Window::getInstance().dndEvent
		{
			unsafe
			{
				*PATH = (*FILESELECTION, file[0].clone());
				exec(String::from("reload"));
			}
		}

		if let Some(act) = maction
		{
			if act == Action::MoveCam
			{
				cam.getTransformable().translate(glam::vec2(
					(mx - mpos.0) * scale,
					(my - mpos.1) * scale
				));
			}
		}
		
		cam.clear();
		cam.toggleTransform(true);
		Shapes::line(
			glam::vec2(-1000.0, 0.0),
			glam::vec2(1000.0, 0.0),
			glam::Vec4::splat(1.0),
			glam::Vec4::splat(1.0)
		);
		Shapes::line(
			glam::vec2(0.0, -1000.0),
			glam::vec2(0.0, 1000.0),
			glam::Vec4::splat(1.0),
			glam::Vec4::splat(1.0)
		);
		unsafe
		{
			(*SKELETON).update();
			cam.draw(&mut *SKELETON);
		}
		cam.toggleTransform(false);
		cam.display();
		cam.draw(Window::getUI());
		Window::display();
		mpos = (mx, my);
	}
}