use std::{io::{ErrorKind, Read, Write}, net::{TcpStream, UdpSocket}};

use mlua::{Error, Lua, Table};

use super::Window::Window;

pub struct Network
{
	tcp: Option<TcpStream>,
	udp: Option<UdpSocket>,
	name: String,
	class: String,
	id: u8,
	pub order: u32,
}

impl Network
{
	pub fn new() -> Self
	{
		Self
		{
			tcp: None,
			udp: None,
			name: String::new(),
			class: String::new(),
			id: 0,
			order: 0,
		}
	}

	pub fn initLua(script: &Lua)
	{
		let table = script.create_table().unwrap();

		table.set("connectTCP", script.create_function(Network::connectTCP).unwrap());
		table.set("connectUDP", script.create_function(Network::connectUDP).unwrap());
		table.set("receiveTCP", script.create_function(Network::receiveTCP).unwrap());
		table.set("receiveUDP", script.create_function(Network::receiveUDP).unwrap());
		table.set("getName", script.create_function(Network::getName).unwrap());
		table.set("setName", script.create_function(Network::setName).unwrap());
		table.set("getID", script.create_function(Network::getID).unwrap());
		table.set("setID", script.create_function(Network::setID).unwrap());
		table.set("getClass", script.create_function(Network::getClass).unwrap());
		table.set("setClass", script.create_function(Network::setClass).unwrap());
		table.set("sendTextTCP", script.create_function(Network::sendTextTCP).unwrap());
		table.set("sendStateUDP", script.create_function(Network::sendStateUDP).unwrap());
		table.set("parseID", script.create_function(Network::parseID).unwrap());
		table.set("bytesToNum", script.create_function(Network::bytesToNum).unwrap());
		table.set("parseState", script.create_function(Network::parseState).unwrap());

		script.globals().set("network", table);
	}

	pub fn connectTCP(_: &Lua, addr: String) -> Result<bool, Error>
	{
		let net = Window::getNetwork();
		let tcp = TcpStream::connect(addr);
		if tcp.is_err()
		{
			println!("Failed to connect TCP: {:?}", tcp.unwrap_err());
			return Ok(false);
		}
		let tcp = tcp.unwrap();
		tcp.set_nonblocking(true);
		
		let udp = UdpSocket::bind("0.0.0.0:0");
		if udp.is_err()
		{
			println!("Failed to bind UDP: {:?}", udp.unwrap_err());
			return Ok(false);
		}
		let udp = udp.unwrap();
		udp.set_nonblocking(true);

		net.tcp = Some(tcp);
		net.udp = Some(udp);
		Ok(true)
	}

	pub fn connectUDP(_: &Lua, port: (u8, u8)) -> Result<(), Error>
	{
		let ip = Window::getNetwork().tcp.as_mut().unwrap().peer_addr().unwrap().ip();
		let addr = ip.to_string() + ":" + &u16::from_le_bytes([port.0, port.1]).to_string();
		Window::getNetwork().udp.as_mut().unwrap().connect(addr);
		Ok(())
	}

	pub fn receiveTCP(script: &Lua, _: ()) -> Result<(i32, Table), Error>
	{
		let table = script.create_table().unwrap();
		let tcp = &mut Window::getNetwork().tcp;
		if tcp.is_none() { return Ok((0, table)); }
		let tcp = tcp.as_mut().unwrap();

		let buffer = &mut [0u8; 1024];
		match tcp.read(buffer)
		{
			Ok(size) =>
			{
				for i in 1..size
				{
					table.raw_push(buffer[i]);
				}
				Ok((buffer[0] as i32, table))
			},
			Err(x) =>
			{
				match x.kind()
				{
					ErrorKind::WouldBlock => {},
					ErrorKind::ConnectionRefused =>
					{
						Window::getNetwork().tcp = None;
					}
					_ => {}
				}
				Ok((0, table))
			}
		}
	}

	pub fn receiveUDP(script: &Lua, _: ()) -> Result<Table, Error>
	{
		let table = script.create_table().unwrap();

		let udp = &mut Window::getNetwork().udp;
		if udp.is_none() { return Ok(table); }
		let udp = udp.as_mut().unwrap();

		let buffer = &mut [0u8; 1024];
		let mut result = udp.recv(buffer);
		let mut size = 0;
		while !result.is_err()
		{
			size = result.unwrap();
			result = udp.recv(buffer);
		}
		let err = result.unwrap_err();
		match err.kind()
		{
			ErrorKind::WouldBlock => {},
			ErrorKind::ConnectionRefused =>
			{
				Window::getNetwork().udp = None;
			},
			_ => println!("UDP: {:?}", err)
		}

		for i in 0..size
		{
			table.raw_push(buffer[i]);
		}
		Ok(table)
	}

	pub fn sendStateUDP(_: &Lua, data: (i8, bool, bool, bool, f32, f32)) -> Result<(), Error>
	{
		let net = Window::getNetwork();
		if net.udp.is_none() { return Ok(()); }
		let id = Window::getNetwork().id;
		let velX = data.0;
		let jump = data.1;
		let attack = data.2;
		let protect = data.3;
		let mut state = id - 1;
		if velX == -1 { state = state | 0b00_10_00_00; }
		if velX == 1  { state = state | 0b00_01_00_00; }
		if jump { state = state | 0b00_00_10_00; }
		if attack { state = state | 0b10_00_00_00; }
		if protect { state = state | 0b01_00_00_00; }

		let udp = net.udp.as_mut().unwrap();
		let posX = &data.4.to_le_bytes() as &[u8];
		let posY = &data.5.to_le_bytes() as &[u8];
		udp.send(&[&[state], posX, posY].concat());
		Ok(())
	}

	pub fn sendTextTCP(_: &Lua, data: (u8, String)) -> Result<(), Error>
	{
		let tcp = &mut Window::getNetwork().tcp;
		if tcp.is_none() { return Ok(()); }
		let tcp = tcp.as_mut().unwrap();
		let _ = tcp.write(&[&[data.0], data.1.as_bytes()].concat());
		Ok(())
	}

	pub fn setName(_: &Lua, name: String) -> Result<(), Error>
	{
		Window::getNetwork().name = name;
		Ok(())
	}

	pub fn getName(_: &Lua, _: ()) -> Result<String, Error>
	{
		Ok(Window::getNetwork().name.clone())
	}

	pub fn setID(_: &Lua, id: i32) -> Result<(), Error>
	{
		Window::getNetwork().id = id as u8;
		Ok(())
	}

	pub fn getID(_: &Lua, _: ()) -> Result<i32, Error>
	{
		Ok(Window::getNetwork().id as i32)
	}

	pub fn setClass(_: &Lua, class: String) -> Result<(), Error>
	{
		Window::getNetwork().class = class;
		Ok(())
	}

	pub fn getClass(_: &Lua, _: ()) -> Result<String, Error>
	{
		Ok(Window::getNetwork().class.clone())
	}

	pub fn parseID(_: &Lua, raw: u8) -> Result<u8, Error>
	{
		Ok((raw & 0b00_00_01_11) + 1)
	}

	pub fn bytesToNum(_: &Lua, raw: (u8, u8, u8, u8)) -> Result<f64, Error>
	{
		Ok(f32::from_le_bytes([raw.0, raw.1, raw.2, raw.3]) as f64)
	}

	/*
		00 00 00 00 - P1 стоит на месте
		00 00 00 01 - P2 стоит на месте
		00 00 00 10 - P3 стоит на месте
		00 00 00 11 - P4 стоит на месте
		00 00 01 00 - P5 стоит на месте

		00 10 00 00 - P1 движется влево
		00 01 00 00 - Р1 движется вправо
		00 00 10 00 - Р1 прыгнул

		10 00 00 00 - Р1 атакует текущим оружием
		01 00 00 00 - Р1 встал в защиту

		ServerMessage::PlayerWeaponChanged(String) - игрок поменял текущее оружие
	*/

	pub fn parseState(_: &Lua, raw: u8) -> Result<(i8, bool, bool, bool), Error>
	{
		let moveLeft = (raw & 0b00_10_00_00) != 0;
		let moveRight = (raw & 0b00_01_00_00) != 0;
		let jump = (raw & 0b00_00_10_00) != 0;
		let attack = (raw & 0b10_00_00_00) != 0;
		let protect = (raw & 0b01_00_00_00) != 0;

		Ok((
			if moveLeft { -1 } else if moveRight { 1 } else { 0 },
			jump, attack, protect
		))
	}
}