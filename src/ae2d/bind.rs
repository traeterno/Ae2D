use std::{io::Write, net::{TcpStream, UdpSocket}};

use mlua::{Lua, Table};

use crate::{ae2d::{Entity::Entity, Network::{Network, PlayerState}, Programmable::Variable, Transformable::Transformable2D, World::World}, server::{State::Account, Transmission::ClientMessage}};

use super::{Sprite::Sprite, Text::Text, Window::Window};

fn getSprite(id: String) -> &'static mut Sprite
{
	let mut id = id.split("_");

	match id.nth(0).unwrap()
	{
		"ui" => Window::getUI().getObject(id.nth(0).unwrap().to_string()).getSprite(),
		"ent" => Window::getWorld().getEntity(id.nth(0).unwrap().to_string()).getSprite(),
		x => panic!("Sprite Lua: {x} not defined")
	}
}

fn getText(id: String) -> &'static mut Text
{
	let mut id = id.split("_");
	
	match id.nth(0).unwrap()
	{
		"ui" => Window::getUI().getObject(id.nth(0).unwrap().to_string()).getText(),
		x => panic!("Text Lua: {x} not defined")
	}
}

fn getEntity(s: &Lua) -> &'static mut Entity
{
	let id: String = s.globals().get("ScriptID").unwrap();
	Window::getWorld().getEntity(
		id.split("_").nth(1).unwrap().to_string()
	)
}

pub fn execFunc(script: &Lua, func: &str)
{
	if let Ok(f) = script.globals().get::<mlua::Function>(func)
	{
		match f.call::<mlua::Value>(())
		{
			Ok(_) => {}
			Err(x) => { println!("{x}"); }
		}
	}
}

pub fn sprite(s: &Lua)
{
	let t = s.create_table().unwrap();

	let _ = t.set("draw",
	s.create_function(|s, _: ()|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		Window::getCamera().draw(spr);
		Ok(())
	}).unwrap());

	let _ = t.set("size",
	s.create_function(|s, _: ()|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		let s = spr.getFrameSize();
		Ok((s.x, s.y))
	}).unwrap());

	let _ = t.set("texSize",
	s.create_function(|s, _: ()|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		let s = spr.getTexSize();
		Ok((s.x, s.y))
	}).unwrap());

	let _ = t.set("bounds",
	s.create_function(|s, _: ()|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		let s = spr.getBounds();
		Ok((s.x, s.y, s.z, s.w))
	}).unwrap());

	let _ = t.set("setTextureRect",
	s.create_function(|s, x: (f32, f32, f32, f32)|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		spr.setTextureRect(glam::vec4(x.0, x.1, x.2, x.3));
		Ok(())
	}).unwrap());

	let _ = t.set("setAnimation",
	s.create_function(|s, x: String|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		spr.setAnimation(x);
		Ok(())
	}).unwrap());

	let _ = t.set("loadAnimation",
	s.create_function(|s, x: String|
	{
		*getSprite(s.globals().raw_get("ScriptID").unwrap()) = Sprite::animated(x);
		Ok(())
	}).unwrap());

	let _ = t.set("loadImage",
	s.create_function(|s, x: String|
	{
		*getSprite(s.globals().raw_get("ScriptID").unwrap()) = Sprite::image(x);
		Ok(())
	}).unwrap());

	let _ = t.set("setColor",
	s.create_function(|s, x: (u8, u8, u8, u8)|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		spr.setColor(x);
		Ok(())
	}).unwrap());

	let _ = t.set("setPosition",
	s.create_function(|s, x: (f32, f32)|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		spr.getTransformable().setPosition(glam::vec2(x.0, x.1));
		Ok(())
	}).unwrap());

	let _ = t.set("translate",
	s.create_function(|s, x: (f32, f32)|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		spr.getTransformable().translate(glam::vec2(x.0, x.1));
		Ok(())
	}).unwrap());

	let _ = t.set("getPosition",
	s.create_function(|s, _: ()|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		let x = spr.getTransformable().getPosition();
		Ok((x.x, x.y))
	}).unwrap());

	let _ = t.set("setOrigin",
	s.create_function(|s, x: (f32, f32)|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		spr.getTransformable().setOrigin(glam::vec2(x.0, x.1));
		Ok(())
	}).unwrap());

	let _ = t.set("getOrigin",
	s.create_function(|s, _: ()|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		let x = spr.getTransformable().getOrigin();
		Ok((x.x, x.y))
	}).unwrap());

	let _ = t.set("setScale",
	s.create_function(|s, x: (f32, f32)|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		spr.getTransformable().setScale(glam::vec2(x.0, x.1));
		Ok(())
	}).unwrap());

	let _ = t.set("scale",
	s.create_function(|s, x: (f32, f32)|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		spr.getTransformable().scale(glam::vec2(x.0, x.1));
		Ok(())
	}).unwrap());

	let _ = t.set("getScale",
	s.create_function(|s, _: ()|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		let x = spr.getTransformable().getScale();
		Ok((x.x, x.y))
	}).unwrap());

	let _ = t.set("setRotation",
	s.create_function(|s, x: f32|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		spr.getTransformable().setRotation(x);
		Ok(())
	}).unwrap());

	let _ = t.set("rotate",
	s.create_function(|s, x: f32|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		spr.getTransformable().rotate(x);
		Ok(())
	}).unwrap());

	let _ = t.set("getRotation",
	s.create_function(|s, _: ()|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		let x = spr.getTransformable().getRotation();
		Ok(x)
	}).unwrap());

	let _ = t.set("applyModel",
	s.create_function(|s, shader: String|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		let s = Window::getShader(shader);
		s.activate();
		s.setMat4("model", spr.getTransformable().getMatrix());
		Ok(())
	}).unwrap());

	let _ = t.set("getCurrentFrame",
	s.create_function(|s, _: ()|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		let f = spr.getCurrentFrame();
		Ok((f.x, f.y, f.z, f.w))
	}).unwrap());

	let _ = t.set("bindTexture",
	s.create_function(|s, _: ()|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		unsafe
		{
			gl::BindTexture(gl::TEXTURE_2D, spr.getTexture());
		}
		Ok(())
	}).unwrap());

	let _ = t.set("tickAnimation",
	s.create_function(|s, _: ()|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		spr.update();
		Ok(())
	}).unwrap());

	let _ = s.globals().set("sprite", t);
}

pub fn text(s: &Lua)
{
	let t = s.create_table().unwrap();

	let _ = t.set("draw",
	s.create_function(|s, _: ()|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		Window::getCamera().draw(txt);
		Ok(())
	}).unwrap());

	let _ = t.set("size",
	s.create_function(|s, _: ()|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		let d = txt.getDimensions();
		Ok((d.x, d.y))
	}).unwrap());

	let _ = t.set("bounds",
	s.create_function(|s, _: ()|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		let d = txt.getBounds();
		Ok((d.x, d.y, d.z, d.w))
	}).unwrap());

	let _ = t.set("setString",
	s.create_function(|s, x: String|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		txt.setString(x);
		Ok(())
	}).unwrap());

	let _ = t.set("getString",
	s.create_function(|s, _: ()|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		Ok(txt.getString())
	}).unwrap());

	let _ = t.set("setColor",
	s.create_function(|s, x: (u8, u8, u8, u8)|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		txt.setColor(glam::vec4(
			x.0 as f32 / 255.0,
			x.1 as f32 / 255.0,
			x.2 as f32 / 255.0,
			x.3 as f32 / 255.0
		));
		Ok(())
	}).unwrap());

	let _ = t.set("getColor",
	s.create_function(|s, _: ()|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		let c = txt.getColor();
		Ok((c.x, c.y, c.z, c.w))
	}).unwrap());

	let _ = t.set("setPosition",
	s.create_function(|s, x: (f32, f32)|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		txt.getTransformable().setPosition(glam::vec2(x.0, x.1));
		Ok(())
	}).unwrap());

	let _ = t.set("translate",
	s.create_function(|s, x: (f32, f32)|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		txt.getTransformable().translate(glam::vec2(x.0, x.1));
		Ok(())
	}).unwrap());

	let _ = t.set("getPosition",
	s.create_function(|s, _: ()|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		let x = txt.getTransformable().getPosition();
		Ok((x.x, x.y))
	}).unwrap());

	let _ = t.set("setOrigin",
	s.create_function(|s, x: (f32, f32)|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		txt.getTransformable().setOrigin(glam::vec2(x.0, x.1));
		Ok(())
	}).unwrap());

	let _ = t.set("getOrigin",
	s.create_function(|s, _: ()|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		let x = txt.getTransformable().getOrigin();
		Ok((x.x, x.y))
	}).unwrap());

	let _ = t.set("setScale",
	s.create_function(|s, x: (f32, f32)|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		txt.getTransformable().setScale(glam::vec2(x.0, x.1));
		Ok(())
	}).unwrap());

	let _ = t.set("scale",
	s.create_function(|s, x: (f32, f32)|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		txt.getTransformable().scale(glam::vec2(x.0, x.1));
		Ok(())
	}).unwrap());

	let _ = t.set("getScale",
	s.create_function(|s, _: ()|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		let x = txt.getTransformable().getScale();
		Ok((x.x, x.y))
	}).unwrap());

	let _ = t.set("setRotation",
	s.create_function(|s, x: f32|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		txt.getTransformable().setRotation(x);
		Ok(())
	}).unwrap());

	let _ = t.set("rotate",
	s.create_function(|s, x: f32|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		txt.getTransformable().rotate(x);
		Ok(())
	}).unwrap());

	let _ = t.set("getRotation",
	s.create_function(|s, _: ()|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		let x = txt.getTransformable().getRotation();
		Ok(x)
	}).unwrap());

	let _ = s.globals().set("text", t);
}

pub fn network(s: &Lua)
{
	let t = s.create_table().unwrap();

	let _ = t.raw_set("id",
	s.create_function(|_, _: ()|
	{
		Ok(Window::getNetwork().id)
	}).unwrap());

	let _ = t.raw_set("connect",
	s.create_function(|_, addr: String|
	{
		let net = Window::getNetwork();

		let tcp = TcpStream::connect_timeout(
			&addr.parse().unwrap(),
			std::time::Duration::from_secs(5)
		);
		if tcp.is_err()
		{
			println!("TCP failed: {}", tcp.unwrap_err());
			return Ok(false);
		}
		let tcp = tcp.unwrap();
		let _ = tcp.set_nodelay(true);

		let udp = UdpSocket::bind("0.0.0.0:0");
		if udp.is_err()
		{
			println!("UDP failed: {}", udp.unwrap_err());
			return Ok(false);
		}
		let udp = udp.unwrap();
		let _ = udp.set_nonblocking(true);

		net.tcp = Some(tcp);
		net.udp = Some(udp);
		std::thread::spawn(Network::tcpThread);
		Ok(true)
	}).unwrap());

	let _ = t.raw_set("disconnect",
	s.create_function(|_, _: ()|
	{
		let net = Window::getNetwork();
		net.id = 0;
		net.tcp = None;
		net.udp = None;
		net.avatars.clear();
		net.state.clear();
		Ok(())
	}).unwrap());

	let _ = t.raw_set("login",
	s.create_function(|_, data: (u8, String)|
	{
		let net = Window::getNetwork();
		net.id = data.0;
		net.avatars.insert(data.0, Account
		{
			name: data.1,
			..Default::default()
		});
		Ok(())
	}).unwrap());

	let _ = t.raw_set("setState",
	s.create_function(|_, x: (f32, f32, f32, f32, Table)|
	{
		Window::getNetwork().setState(PlayerState
		{
			pos: (x.0, x.1),
			vel: (x.2, x.3),
			moveX: x.4.raw_get("MoveX").unwrap(),
			jump: x.4.raw_get("Jump").unwrap(),
			attack: x.4.raw_get("Attack").unwrap(),
			protect: x.4.raw_get("Protect").unwrap(),
			updated: true
		});
		Ok(())
	}).unwrap());

	let _ = t.raw_set("getState",
	s.create_function(|_, id: u8|
	{
		if id == 0
		{
			return Ok((
				0.0, 0.0, 0.0, 0.0,
				0, false, false, false
			));
		}

		let net = Window::getNetwork();
		let s = &mut net.state[(id - 1) as usize];
		s.updated = false;
		Ok((
			s.pos.0, s.pos.1, s.vel.0, s.vel.1,
			s.moveX, s.jump, s.attack, s.protect
		))
	}).unwrap());

	let _ = t.raw_set("hasMessage",
	s.create_function(|_, id: u8|
	{
		for msg in &Window::getNetwork().tcpHistory
		{
			match msg
			{
				ClientMessage::Login(..) => if id == 1 { return Ok(true) }
				ClientMessage::Disconnected(..) => if id == 2 { return Ok(true) }
				ClientMessage::Chat(..) => if id == 3 { return Ok(true) }
				ClientMessage::GameInfo(..) => if id == 4 { return Ok(true) }
				ClientMessage::PlayerInfo(..) => if id == 5 { return Ok(true) }
			}
		}
		Ok(false)
	}).unwrap());

	let _ = t.raw_set("getMessage",
	s.create_function(|s, msg: u8|
	{
		let t = s.create_table().unwrap();
		let net = Window::getNetwork();

		for i in 0..net.tcpHistory.len()
		{
			let found: bool;
			match &net.tcpHistory[i]
			{
				ClientMessage::Login(id, name) =>
				{
					if msg != 1 { continue; }
					let _ = t.raw_set("id", *id);
					let _ = t.raw_set("name", name.clone());
					found = true;
				}
				ClientMessage::Disconnected(id) =>
				{
					if msg != 2 { continue; }
					let _ = t.raw_set("id", *id);
					found = true;
				}
				ClientMessage::Chat(message) =>
				{
					if msg != 3 { continue; }
					let _ = t.raw_set("msg", message.clone());
					found = true;
				}
				ClientMessage::GameInfo(kind, info) =>
				{
					if msg != 4 { continue; }
					let _ = t.raw_set("kind", *kind);
					match kind
					{
						0 =>
						{
							let _ = t.raw_set("playersCount", info[0]);
							let _ = t.raw_set("tickRate", info[1]);
							let _ = t.raw_set("maxItemCellSize", info[2]);
							let _ = t.raw_set(
								"udp",
								u16::from_be_bytes([info[3], info[4]])
							);
						}
						1 =>
						{
							let _ = t.raw_set("ready", info[0]);
						}
						2 => {}
						x =>
						{
							println!("Unknown GameInfo: #{x}");
						}
					}
					found = true;
				}
				ClientMessage::PlayerInfo(id, info, raw) =>
				{
					if msg != 5 { continue; }
					let _ = t.raw_set("id", *id);
					let _ = t.raw_set("kind", *info);
					match *info
					{
						0 => t.raw_set(
							"name", String::from_utf8_lossy(&raw).to_string()
						).unwrap(),
						1 => t.raw_set(
							"class", String::from_utf8_lossy(&raw).to_string()
						).unwrap(),
						2 =>
						{
							let _ = t.raw_set("clrR", raw[0]);
							let _ = t.raw_set("clrG", raw[1]);
							let _ = t.raw_set("clrB", raw[2]);
						}
						3 => t.raw_set(
							"hp", u16::from_be_bytes([raw[0], raw[1]])
						).unwrap(),
						x => println!("Invalid PlayerInfo: {x}")
					}
					found = true;
				}
			}
			if found { net.tcpHistory.swap_remove(i); break; }
		}
		
		Ok(t)
	}).unwrap());

	let _ = t.raw_set("sendMessage",
	s.create_function(|_, x: (u8, Table)|
	{
		let tcp = Window::getNetwork().tcp.as_mut().unwrap();
		if let Err(x) = tcp.write(&match x.0
		{
			1 =>
			{
				let msg: String = x.1.get("msg").unwrap();
				[&[1u8], msg.as_bytes(), &[0u8]].concat()
			}
			2 =>
			{
				vec![2u8]
			}
			3 =>
			{
				vec![
					3u8,
					x.1.get("kind").unwrap()
				]
			}
			4 =>
			{
				vec![
					4u8,
					x.1.get("id").unwrap(),
					x.1.get("kind").unwrap()
				]
			}
			5 =>
			{
				if let Some(name) = x.1
					.raw_get::<mlua::Value>("name").unwrap().as_string_lossy()
				{
					[
						&[5u8], &[0u8],
						name.as_bytes(), &[0u8]
					].concat()
				}
				else if let Some(class) = x.1
					.raw_get::<mlua::Value>("class").unwrap().as_string_lossy()
				{
					[
						&[5u8], &[1u8],
						class.as_bytes(), &[0u8]
					].concat()
				}
				else if let Some(color) = x.1
					.raw_get::<mlua::Value>("color").unwrap().as_table()
				{
					vec![
						5u8, 2u8,
						color.raw_get("r").unwrap(),
						color.raw_get("g").unwrap(),
						color.raw_get("b").unwrap(),
						0u8
					]
				}
				else if let Some(hp) = x.1
					.raw_get::<mlua::Value>("hp").unwrap().as_u32()
				{
					[
						&[5u8], &[3u8],
						&(hp as u16).to_be_bytes() as &[u8],
						&[0u8]
					].concat()
				}
				else { vec![] }
			}
			6 =>
			{
				if let Some(start) = x.1
					.raw_get::<mlua::Value>("start").unwrap().as_boolean()
				{
					vec![
						6u8, 0u8,
						start as u8, 0u8
					]
				}
				else if let Some(save) = x.1
					.raw_get::<mlua::Value>("save").unwrap().as_string_lossy()
				{
					[
						&[6u8], &[1u8],
						save.as_bytes(),
						&[0u8]
					].concat()
				}
				else { vec![] }
			}
			x =>
			{
				println!("Unknown ServerMessage: {x}");
				vec![]
			}
		}) { println!("Sending error: {x:?}"); }
		Ok(())
	}).unwrap());

	let _ = t.raw_set("serverIP",
	s.create_function(|_, _: ()|
	{
		Ok(Window::getNetwork().tcp.as_mut().unwrap().peer_addr().unwrap().to_string())
	}).unwrap());
	
	let _ = t.raw_set("findServer",
	s.create_function(|_, _: ()|
	{
		let s = UdpSocket::bind("0.0.0.0:0").unwrap();
		let mut buffer = [0u8; 256];
		let _ = s.set_broadcast(true);
		let _ = s.set_read_timeout(Some(std::time::Duration::from_secs(1)));
		for _ in 0..10
		{
			let _ = s.send_to(&[], "255.255.255.255:26225");
			match s.recv_from(&mut buffer)
			{
				Ok((_, addr)) =>
				{
					let tcp = u16::from_be_bytes([buffer[0], buffer[1]]);
					return Ok(addr.ip().to_string() + ":" + &tcp.to_string());
				}
				Err(x) => match x.kind()
				{
					std::io::ErrorKind::WouldBlock => {}
					std::io::ErrorKind::TimedOut => {}
					_ => println!("Список серверов не был получен: {x:?}")
				}
			}
		}
		Ok(String::new())
	}).unwrap());

	let _ = t.raw_set("playersCount",
	s.create_function(|_, _: ()|
	{
		Ok(Window::getNetwork().avatars.len())
	}).unwrap());

	let _ = t.raw_set("stateReady",
	s.create_function(|_, id: u8|
	{
		Ok(Window::getNetwork().state[(id - 1) as usize].updated)
	}).unwrap());

	let _ = t.raw_set("setup",
	s.create_function(|_, x: (u16, u8, Table)|
	{
		Window::getNetwork().setup(x.0, x.1, x.2);
		Ok(())
	}).unwrap());

	let _ = t.raw_set("setPlayersCount",
	s.create_function(|_, x: u8|
	{
		Window::getNetwork().setPlayersCount(x);
		Ok(())
	}).unwrap());

	let _ = t.raw_set("getEP",
	s.create_function(|_, _: ()|
	{
		Ok(Window::getNetwork().getEP())
	}).unwrap());

	let _ = t.raw_set("getPlayer",
	s.create_function(|s, id: u8|
	{
		let p = s.create_table().unwrap();
		let a = Window::getNetwork().avatars.get(&id).unwrap();
		let _ = p.raw_set("name", a.name.clone());
		let _ = p.raw_set("class", a.class.clone());
		let _ = p.raw_set("clrR", a.color.0);
		let _ = p.raw_set("clrG", a.color.1);
		let _ = p.raw_set("clrB", a.color.2);
		let _ = p.raw_set("hp", a.hp);
		Ok(p)
	}).unwrap());

	let _ = t.raw_set("setPlayerInfo",
	s.create_function(|_, x: (u8, mlua::Table)|
	{
		let a = Window::getNetwork().avatars.get_mut(&x.0).unwrap();
		for res in x.1.pairs::<String, mlua::Value>()
		{
			if let Ok((k, p)) = res
			{
				if k == "name" { a.name = p.as_string_lossy().unwrap(); }
				if k == "class" { a.class = p.as_string_lossy().unwrap(); }
				if k == "clrR" { a.color.0 = p.as_u32().unwrap() as u8; }
				if k == "clrG" { a.color.1 = p.as_u32().unwrap() as u8; }
				if k == "clrB" { a.color.2 = p.as_u32().unwrap() as u8; }
				if k == "hp" { a.hp = p.as_u32().unwrap() as u16; }
			}
		}
		Ok(())
	}).unwrap());

	let _ = s.globals().set("network", t);
}

pub fn window(script: &Lua)
{
	let table = script.create_table().unwrap();

	let _ = table.raw_set("launchServer",
	script.create_function(|_, _: ()|
	{
		Window::launchServer();
		Ok(())
	}).unwrap());

	let _ = table.raw_set("size",
	script.create_function(|_, _: ()|
	{
		Ok(Window::getSize())
	}).unwrap());
	
	let _ = table.raw_set("clearCache",
	script.create_function(|_, _: ()|
	{
		Window::clearCache();
		Ok(())
	}).unwrap());
	
	let _ = table.raw_set("resetDT",
	script.create_function(|_, _: ()|
	{
		Window::resetDT();
		Ok(())
	}).unwrap());

	let _ = table.raw_set("dt",
	script.create_function(|_, _: ()|
	{
		Ok(Window::getDeltaTime())
	}).unwrap());

	let _ = table.raw_set("getNum",
	script.create_function(|_, name: String|
	{
		Ok(Window::getInstance().prog.get(&name)
			.unwrap_or(&Variable::default()).num)
	}).unwrap());

	let _ = table.raw_set("getStr",
	script.create_function(|_, name: String|
	{
		Ok(Window::getInstance().prog.get(&name)
			.unwrap_or(&Variable::default()).string.clone())
	}).unwrap());
	
	let _ = table.raw_set("setNum",
	script.create_function(|_, x: (String, f32)|
	{
		Window::getInstance().prog.insert(
			x.0,
			Variable { num: x.1, string: String::new() }
		);
		Ok(())
	}).unwrap());
	
	let _ = table.raw_set("setStr",
	script.create_function(|_, x: (String, String)|
	{
		Window::getInstance().prog.insert(
			x.0,
			Variable { num: 0.0, string: x.1 }
		);
		Ok(())
	}).unwrap());

	let _ = table.raw_set("mousePos",
	script.create_function(|_, _: ()|
	{
		Ok(Window::getInstance().window.as_ref().unwrap().get_cursor_pos())
	}).unwrap());

	let _ = table.raw_set("setMousePos",
	script.create_function(|_, x: (f32, f32)|
	{
		Window::setMousePos(glam::vec2(x.0, x.1));
		Ok(())
	}).unwrap());
	
	let _ = table.raw_set("mousePressed",
	script.create_function(|_, name: String|
	{
		Ok(Window::getInstance().window.as_ref().unwrap()
			.get_mouse_button(Window::strToMB(name)) == glfw::Action::Press)
	}).unwrap());

	let _ = table.raw_set("mouseJustPressed",
	script.create_function(|_, name: String|
	{
		let e = Window::getInstance().mouseEvent;
		if e.is_none() { return Ok(false); }
		let e = e.unwrap();
		Ok(e.0 == Window::strToMB(name) && e.1 == glfw::Action::Press)
	}).unwrap());
	
	let _ = table.raw_set("keyPressed",
	script.create_function(|_, name: String|
	{
		Ok(Window::getInstance().window.as_ref().unwrap()
			.get_key(Window::strToKey(name)) == glfw::Action::Press)
	}).unwrap());
	
	let _ = table.raw_set("keyJustPressed",
	script.create_function(|_, name: String|
	{
		let e = Window::getInstance().keyEvent;
		if e.is_none() { return Ok(false); }
		let e = e.unwrap();
		Ok(
			e.0 == Window::strToKey(name) &&
			(e.1 == glfw::Action::Press || e.1 == glfw::Action::Repeat)
		)
	}).unwrap());
	
	let _ = table.raw_set("keyModPressed",
	script.create_function(|_, name: String|
	{
		let e = Window::getInstance().keyEvent;
		if e.is_none() { return Ok(false); }
		Ok(e.unwrap().2.intersects(Window::strToMod(name)))
	}).unwrap());

	let _ = table.raw_set("close",
	script.create_function(|_, _: ()|
	{
		Window::close(); Ok(())
	}).unwrap());
	
	let _ = table.raw_set("execute",
	script.create_function(|s, code: String|
	{
		let _ = s.load(code).exec();
		Ok(())
	}).unwrap());

	let _ = table.set("loadUI",
	script.create_function(|_, path: String|
	{
		Window::getUI().requestLoad(path);
		Ok(())
	}).unwrap());

	let _ = table.set("uiSize",
	script.create_function(|_, _: ()|
	{
		let s = Window::getUI().getSize();
		Ok((s.x, s.y))
	}).unwrap());

	let _ = table.raw_set("input",
	script.create_function(|_, _: ()|
	{
		let x = Window::getInstance().inputEvent;
		if let Some(c) = x { Ok(c.to_string()) }
		else { Ok(String::new()) }
	}).unwrap());

	let _ = table.raw_set("clipboard",
	script.create_function(|_, _: ()|
	{
		Ok(Window::getInstance().window.as_mut().unwrap()
			.get_clipboard_string().unwrap_or_default())
	}).unwrap());

	let _ = table.raw_set("setClipboard",
	script.create_function(|_, x: String|
	{
		Window::getInstance().window.as_mut().unwrap().set_clipboard_string(&x);
		Ok(())
	}).unwrap());

	let _ = table.raw_set("screenToWorld",
	script.create_function(|_, x: (f32, f32)|
	{
		let a = Window::getCamera().getTransformable().getPosition();
		let s1 = Window::getCamera().getSize();
		let s2 = Window::getSize();
		let s3 = glam::vec2(s1.x / s2.0 as f32, s1.y / s2.1 as f32);
		let p = -a - s1 / 2.0 + glam::vec2(x.0 * s3.x, x.1 * s3.y);
		Ok((p.x, p.y))
	}).unwrap());

	let _ = table.raw_set("worldToScreen",
	script.create_function(|_, x: (f32, f32)|
	{
		let t = Window::getCamera().getTransformable();
		let p = t.getMatrix().transform_point3(glam::vec3(x.0, x.1, 0.0));
		let s1 = Window::getCamera().getSize();
		let s2 = Window::getSize();
		let s3 = glam::vec2(s2.0 as f32 / s1.x, s2.1 as f32 / s1.y);
		Ok((p.x * s3.x, p.y * s3.y))
	}).unwrap());

	let _ = script.globals().raw_set("window", table);
}

pub fn world(script: &Lua)
{
	let t = script.create_table().unwrap();

	let _ = t.raw_set("name",
	script.create_function(|_, _: ()|
	{
		Ok(Window::getWorld().getName())
	}).unwrap());

	let _ = t.raw_set("load",
	script.create_function(|_, path: String|
	{
		Window::getWorld().load(path);
		Ok(())
	}).unwrap());

	let _ = t.raw_set("parse",
	script.create_function(|_, x: (String, String)|
	{
		Window::getWorld().parse(x.0, x.1);
		Ok(())
	}).unwrap());

	let _ = t.raw_set("spawn",
	script.create_function(|_, data: (String, String, Table)|
	{
		let mut obj = json::object! {};
		for v in data.2.pairs::<String, mlua::Value>()
		{
			if let Err(_) = v { continue; }
			let (var, value) = v.unwrap();
			let _ = if value.is_integer() { obj.insert(&var, value.as_i32().unwrap()) }
			else if value.is_number() { obj.insert(&var, value.as_f32().unwrap()) }
			else if value.is_boolean() { obj.insert(&var, value.as_boolean().unwrap()) }
			else { obj.insert(&var, value.as_string_lossy().unwrap()) };
		}
		Window::getWorld().spawn(data.0, data.1, obj);
		Ok(())
	}).unwrap());

	let _ = t.raw_set("createTrigger",
	script.create_function(|_, x: (String, String, f32, f32, f32, f32)|
	{
		Window::getWorld().createTrigger(
			x.0, x.1,
			glam::vec4(x.2, x.3, x.4, x.5
		));
		Ok(())
	}).unwrap());
	
	let _ = t.raw_set("modifyTrigger",
	script.create_function(|_, x: (String, f32, f32, f32, f32)|
	{
		Window::getWorld().modifyTrigger(
			x.0, glam::vec4(x.1, x.2, x.3, x.4)
		);
		Ok(())
	}).unwrap());
	
	let _ = t.raw_set("getTriggers",
	script.create_function(|s, _: ()|
	{
		let t = s.create_table().unwrap();
		for (id, hb) in Window::getWorld().getTriggers()
		{
			let h = s.create_table().unwrap();
			let _ = h.raw_set("name", hb.0.as_str());
			let _ = h.raw_set("x", hb.1.x);
			let _ = h.raw_set("y", hb.1.y);
			let _ = h.raw_set("w", hb.1.z);
			let _ = h.raw_set("h", hb.1.w);
			let _ = t.raw_set(id.clone(), h);
		}
		Ok(t)
	}).unwrap());

	let _ = t.raw_set("setNum",
	script.create_function(|_, data: (String, f32)|
	{
		Window::getWorld().getProgrammable().insert(
			data.0,
			Variable { num: data.1, string: String::new() }
		);
		Ok(())
	}).unwrap());

	let _ = t.raw_set("setStr",
	script.create_function(|_, data: (String, String)|
	{
		Window::getWorld().getProgrammable().insert(
			data.0,
			Variable { num: 0.0, string: data.1 }
		);
		Ok(())
	}).unwrap());

	let _ = t.raw_set("getNum",
	script.create_function(|_, data: String|
	{
		let v = Variable::default();
		Ok(Window::getWorld().getProgrammable().get(&data).unwrap_or(&v).num)
	}).unwrap());
	
	let _ = t.raw_set("getStr",
	script.create_function(|_, data: String|
	{
		let v = Variable::default();
		Ok(Window::getWorld().getProgrammable().get(&data).unwrap_or(&v).string.clone())
	}).unwrap());
	
	let _ = t.raw_set("setCamSize",
	script.create_function(|_, x: (i32, i32)|
	{
		Window::getCamera().setSize(true, x);
		Ok(())
	}).unwrap());
	
	let _ = t.raw_set("setCamPos",
	script.create_function(|_, x: (f32, f32)|
	{
		Window::getCamera().getTransformable().setPosition(glam::vec2(-x.0, -x.1));
		Ok(())
	}).unwrap());
	
	let _ = t.raw_set("getCamPos",
	script.create_function(|_, _: ()|
	{
		let p = Window::getCamera().getTransformable().getPosition();
		Ok((-p.x, -p.y))
	}).unwrap());
	
	let _ = t.raw_set("setCamOrigin",
	script.create_function(|_, x: (f32, f32)|
	{
		Window::getCamera().getTransformable().setOrigin(glam::vec2(-x.0, -x.1));
		Ok(())
	}).unwrap());
	
	let _ = t.raw_set("getCamOrigin",
	script.create_function(|_, _: ()|
	{
		let p = Window::getCamera().getTransformable().getOrigin();
		Ok((-p.x, -p.y))
	}).unwrap());
	
	let _ = t.raw_set("kill",
	script.create_function(|_, x: String|
	{
		Window::getWorld().kill(x);
		Ok(())
	}).unwrap());
	
	let _ = t.raw_set("reset",
	script.create_function(|_, _: ()|
	{
		*Window::getWorld() = World::new();
		Ok(())
	}).unwrap());

	let _ = t.raw_set("setLayersCount",
	script.create_function(|_, x: u8|
	{
		Window::getWorld().setLayersCount(x);
		Ok(())
	}).unwrap());
	
	let _ = script.globals().raw_set("world", t);
}

pub fn shapes(script: &Lua)
{
	let t = script.create_table().unwrap();

	let _ = t.raw_set("rect",
	script.create_function(|_, x: (f32, f32, f32, f32, u8, u8, u8, u8)|
	{
		let s = Window::getShader(String::from("shape"));
		s.activate();
		let mut ts = Transformable2D::new();
		ts.setPosition(glam::vec2(x.0, x.1));
		s.setVec2("size", glam::vec2(x.2, x.3));
		s.setMat4("model", ts.getMatrix());
		s.setVec4("clr", glam::vec4(
			x.4 as f32 / 255.0,
			x.5 as f32 / 255.0,
			x.6 as f32 / 255.0,
			x.7 as f32 / 255.0
		));
		Window::getCamera().drawShape();
		Ok(())
	}).unwrap());

	let _ = t.raw_set("custom",
	script.create_function(|_, _: ()|
	{
		Window::getCamera().drawShape();
		Ok(())
	}).unwrap());

	let _ = script.globals().raw_set("shapes", t);
}

pub fn shaders(script: &Lua)
{
	let t = script.create_table().unwrap();

	let _ = t.raw_set("bind",
	script.create_function(|_, name: String|
	{
		Window::getShader(name).activate();
		Ok(())
	}).unwrap());
	
	let _ = t.raw_set("setInt",
	script.create_function(|_, x: (String, String, i32)|
	{
		Window::getShader(x.0.clone()).setInt(&x.1, x.2);
		Ok(())
	}).unwrap());
	
	let _ = t.raw_set("setVec2",
	script.create_function(|_, x: (String, String, f32, f32)|
	{
		Window::getShader(x.0.clone())
			.setVec2(&x.1, glam::vec2(x.2, x.3));
		Ok(())
	}).unwrap());
	
	let _ = t.raw_set("setVec3",
	script.create_function(|_, x: (String, String, f32, f32, f32)|
	{
		Window::getShader(x.0.clone())
			.setVec3(&x.1, glam::vec3(x.2, x.3, x.4));
		Ok(())
	}).unwrap());
	
	let _ = t.raw_set("setVec4",
	script.create_function(|_, x: (String, String, f32, f32, f32, f32)|
	{
		Window::getShader(x.0.clone())
			.setVec4(&x.1, glam::vec4(x.2, x.3, x.4, x.5));
		Ok(())
	}).unwrap());
	
	let _ = script.globals().raw_set("shaders", t);
}