use std::{io::Write, net::{TcpStream, UdpSocket}};

use mlua::{Lua, Table};

use crate::{ae2d::Network::Network, server::Transmission::ClientMessage};

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

		let tcp = TcpStream::connect(addr);
		if tcp.is_err()
		{
			println!("TCP failed: {}", tcp.unwrap_err());
			return Ok(false);
		}
		let tcp = tcp.unwrap();

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
				ClientMessage::Login(..) => if id == 1 { return Ok(true); }
				ClientMessage::Disconnected(..) => if id == 2 { return Ok(true); }
				ClientMessage::Chat(..) => if id == 3 { return Ok(true); }
				ClientMessage::SetPosition(..) => if id == 4 { return Ok(true); }
				ClientMessage::GetInfo(..) => if id == 5 { return Ok(true); },
				ClientMessage::SelectChar(..) => if id == 6 { return Ok(true); }
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
				ClientMessage::GetInfo(udpPort, tickRate, checkpoints,
					extendPlayers, players) =>
				{
					if msg != 5 { continue; }
					let _ = t.raw_set("udpPort", *udpPort);
					let _ = t.raw_set("tickRate", *tickRate);
					let _ = t.raw_set("extendPlayers", *extendPlayers);
					let c = s.create_table().unwrap();
					for cp in checkpoints { let _ = c.raw_push(cp.clone()); }
					let _ = t.raw_set("checkpoints", c);
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
			4 =>
			{
				[4u8].to_vec()
			}
			5 =>
			{
				let id: u8 = x.1.get("id").unwrap();
				[5u8, id].to_vec()
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

	let _ = s.globals().set("network", t);
}