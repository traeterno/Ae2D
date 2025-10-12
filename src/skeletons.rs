#![allow(non_snake_case, static_mut_refs, dead_code)]

use std::{env, sync::LazyLock};

use crate::ae2d::{Shapes, Skeleton::{Bone, Interpolation, Skeleton}, Window::Window};

mod ae2d;
mod server;

static mut SKELETON: LazyLock<Skeleton> = LazyLock::new(|| Skeleton::new());
static mut RIG: LazyLock<String> = LazyLock::new(|| String::new());
static mut SL: LazyLock<String> = LazyLock::new(|| String::new());
static mut TEX: LazyLock<String> = LazyLock::new(|| String::new());
static mut AL: LazyLock<String> = LazyLock::new(|| String::new());

fn initLua(s: &mlua::Lua)
{
	let _ = s.globals().raw_set("Exec", s.create_function(
		|s, cmd: String| unsafe { Ok(exec(s, cmd)) }
	).unwrap());
}

unsafe fn exec(s: &mlua::Lua, cmd: String) -> mlua::Value
{
	let args: Vec<&str> = cmd.split("|").collect();
	if args[0] == "load"
	{
		if args[1] == "rig"
		{
			*RIG = args[2].to_string();
			(*SKELETON).loadRig((*RIG).clone());
		}
		if args[1] == "sl"
		{
			*SL = args[2].to_string();
			*TEX = (*SKELETON).loadSL((*SL).clone());
		}
		if args[1] == "texture"
		{
			*TEX = std::path::Path::new(args[2])
				.strip_prefix(env::current_dir().unwrap()).unwrap()
				.to_string_lossy().to_string().replace("\\", "/");
			(*SKELETON).loadTexture((*TEX).clone());
		}
		if args[1] == "al"
		{
			*AL = args[2].to_string();
			(*SKELETON).loadAL((*AL).clone());
		}
	}
	if args[0] == "bone"
	{
		let mut path: Vec<&str> = args[2].split("/").collect();
		path.remove(0);
		path.remove(0);
		if let Some(b) = (*SKELETON).getRoot().resolvePath(path.clone())
		{
			match args[1]
			{
				"name" =>
				{
					let n = path.pop().unwrap_or_default();
					if n.is_empty() { return mlua::Value::Nil; }
					let x = (*SKELETON).getRoot().resolvePath(path).unwrap();
					let bl = x.getBones();
					let a = bl.remove(&n.to_string()).unwrap();
					bl.insert(args[3].to_string(), a);
				}
				"angle" => b.angle = args[3].parse().unwrap_or(0.0),
				"length" => b.length = args[3].parse().unwrap_or(0.0),
				"texture" => b.texture = args[3].to_string(),
				"layer" => b.layer = args[3].parse().unwrap_or(0),
				"info" =>
				{
					let t = s.create_table().unwrap();
					let _ = t.raw_set("length", b.length);
					let _ = t.raw_set("angle", b.angle);
					let _ = t.raw_set("texture", b.texture.clone());
					let _ = t.raw_set("layer", b.layer);
					return mlua::Value::Table(t);
				}
				"list" =>
				{
					return mlua::Value::Table((*SKELETON).getRoot().serialize(s));
				}
				"new" =>
				{
					let s = b.getBones().len();
					b.getBones().insert(
						format!("bone{s}"),
						Bone::new()
					);
				}
				"delete" =>
				{
					let n = path.pop().unwrap_or_default();
					if n.is_empty() { return mlua::Value::Nil; }
					if let Some(x) = (*SKELETON).getRoot().resolvePath(path)
					{
						x.getBones().remove(&n.to_string());
					}
				}
				"highlight" => { b.highlight = true; }
				_ => {}
			}
		}
	}
	if args[0] == "sl"
	{
		let sl = (*SKELETON).getSL();
		let spr = sl.get_mut(
			&args.get(2).unwrap_or(&"").to_string()
		);
		match args[1]
		{
			"new" =>
			{ 
				sl.insert(format!("sprite{}", sl.len()),
					(glam::Vec4::ZERO, glam::Vec2::ZERO)
				);
			}
			"delete" =>
			{
				sl.remove(&args[2].to_string());
			}
			"info" => if let Some((r, o)) = spr
			{
				let t = s.create_table().unwrap();
				let _ = t.raw_set("rect", [r.x, r.y, r.z, r.w]);
				let _ = t.raw_set("origin", [o.x, o.y]);
				return mlua::Value::Table(t);
			}
			"list" =>
			{
				let t = s.create_table().unwrap();
				for (n, _) in sl { let _ = t.raw_push(n.clone()); }
				return mlua::Value::Table(t);
			}
			"name" => if let Some((r, o)) = spr.cloned()
			{
				sl.insert(args[3].to_string(), (r, o));
				sl.remove(&args[2].to_string());
			}
			"ox" => if let Some((_, o)) = spr { o.x = args[3].parse().unwrap(); }
			"oy" => if let Some((_, o)) = spr { o.y = args[3].parse().unwrap(); }
			"rx" => if let Some((r, _)) = spr { r.x = args[3].parse().unwrap(); }
			"ry" => if let Some((r, _)) = spr { r.y = args[3].parse().unwrap(); }
			"rw" => if let Some((r, _)) = spr { r.z = args[3].parse().unwrap(); }
			"rh" => if let Some((r, _)) = spr { r.w = args[3].parse().unwrap(); }
			_ => {}
		}
	}
	if args[0] == "files"
	{
		let t = s.create_table().unwrap();
		let _ = t.raw_set("rig", (*RIG).clone());
		let _ = t.raw_set("sl", (*SL).clone());
		let _ = t.raw_set("al", String::new());
		let _ = t.raw_set("tex", (*TEX).clone());
		return mlua::Value::Table(t);
	}
	if args[0] == "save"
	{
		let mut doc = json::object!{};
		println!("Saving {} to {}...", args[1], args[2]);
		if args[1] == "rig"
		{
			let _ = doc.insert("root", (*SKELETON).getRoot().toJSON());
		}
		if args[1] == "sl"
		{
			let mut s = json::object!{};
			for (name, (r, o)) in (*SKELETON).getSL()
			{
				let _ = s.insert(&name, json::object!{
					rect: [r.x, r.y, r.z, r.w],
					offset: [o.x, o.y]
				});
			}
			doc = json::object!{
				texture: (*TEX).clone(),
				sprites: s
			}
		}
		if args[1] == "al"
		{
			// 
		}
		let _ = std::fs::write(args[2], json::stringify(doc));
	}
	if args[0] == "debug"
	{
		(*SKELETON).debug = args[1].parse().unwrap_or(false);
	}
	if args[0] == "anim"
	{
		let anim = (*SKELETON).getCurrentAnimation();
		if args[1] == "toggle"
		{
			(*SKELETON).activeAnim = !(*SKELETON).activeAnim;
		}
		if args[1] == "current"
		{
			let t = s.create_table().unwrap();
			let _ = t.raw_set("name", anim.0);
			if let Some(a) = anim.1
			{
				let _ = t.raw_set("duration", a.calculateDuration());
			}
			let _ = t.raw_set("active", (*SKELETON).activeAnim);
			return mlua::Value::Table(t);
		}
		if args[1] == "tl"
		{
			let mut path = args[2].strip_prefix("/root").unwrap();
			if let Some(p) = path.strip_prefix("/") { path = p; }
			if anim.1.is_none() { return mlua::Value::Nil; }
			if let Some(ref a) = anim.1
			{
				if let Some(x) = a.bones.get(&path.to_string())
				{
					let t = s.create_table().unwrap();
					let _ = t.raw_set("time", x.time);
					let _ = t.raw_set("current", x.current);
					let tl = s.create_table().unwrap();
					for f in &x.frames
					{
						let frame = s.create_table().unwrap();
						let _ = frame.raw_set("timestamp", f.timestamp);
						let _ = frame.raw_set("angleInterpolation", match f.angle.0
						{
							Interpolation::Const => "Const",
							Interpolation::Linear => "Linear",
							Interpolation::CubicIn => "CubicIn",
							Interpolation::CubicOut => "CubicOut",
							Interpolation::CubicInOut => "CubicInOut",
							Interpolation::SineIn => "SineIn",
							Interpolation::SineOut => "SineOut",
							Interpolation::SineInOut => "SineInOut",
						});
						let _ = frame.raw_set("angle", f.angle.1);
						let _ = frame.raw_set("texture", f.texture.clone());
	
						let _ = tl.raw_push(frame);
					}
					let _ = t.raw_set("frames", tl);
					return mlua::Value::Table(t);
				}
			}
		}
		if args[1] == "jump"
		{
			if let Some(a) = anim.1
			{
				for (_, tl) in &mut a.bones
				{
					tl.time = args[2].parse().unwrap_or(0.0);
				}
				a.update((*SKELETON).getRoot());
			}
		}
	}
	mlua::Value::Nil
}

fn main()
{
	Window::init("res/global/se.json");
	let cam = Window::getCamera();

	initLua(Window::getUI().getObject(String::from("toolbox")).getScript());
	initLua(Window::getUI().getObject(String::from("timeline")).getScript());

	unsafe
	{
		let p = &Window::getInstance().prog;
		if let Some(x) = p.get(&String::from("rig"))
		{
			*RIG = x.string.clone();
			(*SKELETON).loadRig((*RIG).clone());
		}
		if let Some(x) = p.get(&String::from("sl"))
		{
			*SL = x.string.clone();
			*TEX = (*SKELETON).loadSL((*SL).clone());
		}
		if let Some(x) = p.get(&String::from("al"))
		{
			*AL = x.string.clone();
			(*SKELETON).loadAL((*AL).clone());
			(*SKELETON).setAnimation(String::from("idle"));
		}
		gl::Enable(gl::BLEND);
	}

	while Window::isOpen()
	{
		Window::update();

		if let Some(e) = Window::getInstance().keyEvent
		{
			if e.0 == glfw::Key::F1 && e.1 == glfw::Action::Press
			{
				Window::getUI().load("res/ui/se.json");
				initLua(Window::getUI().getObject(String::from("toolbox")).getScript());
				initLua(Window::getUI().getObject(String::from("timeline")).getScript());
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
	}
}