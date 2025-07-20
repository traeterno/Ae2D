use std::{io::Write, net::{TcpStream, UdpSocket}};

use mlua::{Lua, Table};

use crate::{ae2d::{Network::Network, Programmable::Variable}, server::Transmission::ClientMessage};

use super::{Sprite::Sprite, Text::Text, Window::Window};

fn getSprite(id: String) -> &'static mut Sprite
{
	let mut id = id.split("_");

	match id.nth(0).unwrap()
	{
		"ui" => Window::getUI().getObject(id.nth(0).unwrap().to_string()).getSprite(),
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

	let _ = t.raw_set("name",
	s.create_function(|_, _: ()|
	{
		Ok(Window::getNetwork().name.clone())
	}).unwrap());

	let _ = t.raw_set("id",
	s.create_function(|_, _: ()|
	{
		Ok(Window::getNetwork().id.clone())
	}).unwrap());

	let _ = t.raw_set("class",
	s.create_function(|_, _: ()|
	{
		Ok(Window::getNetwork().class.clone())
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
		net.class = String::new();
		net.id = 0;
		net.name = String::new();
		net.tcp = None;
		net.udp = None;
		Ok(())
	}).unwrap());

	let _ = t.raw_set("login",
	s.create_function(|_, data: (u8, String, String)|
	{
		let net = Window::getNetwork();
		net.id = data.0;
		net.name = data.1;
		net.class = data.2;
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
		let s = net.state[(id - 1) as usize];
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
				ClientMessage::SetPosition(..) => if id == 4 { return Ok(true) }
				ClientMessage::GetInfo(..) => if id == 5 { return Ok(true) }
				ClientMessage::SelectChar(..) => if id == 6 { return Ok(true) }
				ClientMessage::GameReady(..) => if id == 7 { return Ok(true) }
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
			let mut found = false;
			match &net.tcpHistory[i]
			{
				ClientMessage::Login(id, name, class) =>
				{
					if msg != 1 { continue; }
					let _ = t.raw_set("id", *id);
					let _ = t.raw_set("name", name.clone());
					let _ = t.raw_set("class", class.to_string());
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
				ClientMessage::GetInfo(udpPort, tickRate, checkpoint,
					extendPlayers, players) =>
				{
					if msg != 5 { continue; }
					let _ = t.raw_set("udpPort", *udpPort);
					let _ = t.raw_set("tickRate", *tickRate);
					let _ = t.raw_set("extendPlayers", *extendPlayers);
					let _ = t.raw_set("checkpoint", checkpoint.clone());
					let p = s.create_table().unwrap();
					for pl in players
					{
						let mut a = pl.split("/");
						let b = s.create_table().unwrap();
						let _ = b.raw_set(
							"id", a.nth(0).unwrap().parse::<u8>().unwrap()
						);
						let _ = b.raw_set(
							"name", a.nth(0).unwrap()
						);
						let _ = b.raw_set(
							"class", a.nth(0).unwrap()
						);
						let _ = p.raw_push(b);
					}
					let _ = t.raw_set("players", p);
					found = true;
				},
				ClientMessage::SelectChar(id, class) =>
				{
					if msg != 6 { continue; }
					let _ = t.raw_set("id", *id);
					let _ = t.raw_set("class", class.clone());
					found = true;
				},
				ClientMessage::GameReady(ready) =>
				{
					if msg != 7 { continue; }
					let _ = t.raw_set("ready", *ready);
					found = true;
				}
				_ => {}
			}
			if found { net.tcpHistory.swap_remove(i); break; }
		}
		
		Ok(t)
	}).unwrap());

	let _ = t.raw_set("sendMessage",
	s.create_function(|_, x: (u8, Table)|
	{
		let tcp = Window::getNetwork().tcp.as_mut().unwrap();
		let _ = tcp.write(&match x.0
		{
			1 =>
			{
				let name: String = x.1.get("name").unwrap();
				[&[1u8], name.as_bytes()].concat()
			}
			2 =>
			{
				let msg: String = x.1.get("msg").unwrap();
				[&[2u8], msg.as_bytes()].concat()
			}
			4 =>
			{
				[4u8].to_vec()
			}
			5 =>
			{
				let id: u8 = x.1.get("id").unwrap();
				[5u8, id].to_vec()
			}
			6 =>
			{
				[6u8].to_vec()
			}
			_ => vec![]
		});
		Ok(())
	}).unwrap());

	let _ = t.raw_set("setup", 
	s.create_function(|_, x: (u16, u8)|
	{
		Window::getNetwork().setup(x.0, x.1);
		Ok(())
	}).unwrap());

	let _ = t.raw_set("setEP",
	s.create_function(|_, x: bool|
	{
		Window::getNetwork().setEP(x);
		Ok(())
	}).unwrap());

	let _ = t.raw_set("getEP",
	s.create_function(|_, _: ()|
	{
		Ok(Window::getNetwork().getEP())
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
		let _ = s.set_broadcast(true);
		let _ = s.send_to(&[], "255.255.255.255:26225");
		let mut buffer = [0u8; 256];
		let _ = s.set_read_timeout(Some(std::time::Duration::from_secs(10)));
		match s.recv_from(&mut buffer)
		{
			Ok((_, addr)) =>
			{
				let ip = addr.ip().to_string() + ":" +
					&u16::from_le_bytes([buffer[0], buffer[1]]).to_string();
				return Ok(ip);
			}
			Err(x) => match x.kind()
			{
				std::io::ErrorKind::WouldBlock => {}
				std::io::ErrorKind::TimedOut => {}
				_ => println!("Список серверов не был получен: {x:?}")
			}
		}
		Ok(String::new())
	}).unwrap());

	let _ = s.globals().set("network", t);
}

pub fn window(script: &Lua)
{
	let table = script.create_table().unwrap();

	let _ = table.raw_set("size",
	script.create_function(|_, _: ()|
	{
		Ok(Window::getSize())
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
		Ok(e.0 == Window::strToKey(name) && e.1 == glfw::Action::Press)
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

	let _ = script.globals().set("window", table);

	network(script);
}