use std::{io::{ErrorKind, Read, Write}, net::{TcpStream, UdpSocket}, time::{Duration, Instant}};

use mlua::{Error, Integer, Lua, Number, Table, Value};

use crate::ae2d::Programmable::Variable;

use super::Window::Window;

#[derive(Clone, Copy, Debug)]
struct State
{
	pos: (f32, f32),
	vel: (f32, f32),
	moveX: i8,
	jump: bool,
	attack: bool,
	protect: bool
}

impl State
{
	fn default() -> Self
	{
		Self { pos: (0.0, 0.0), vel: (0.0, 0.0), moveX: 0, jump: false, attack: false, protect: false }
	}
	fn parse(data: &[u8]) -> (u8, Self)
	{
		let state = data[0];
		let id = state & 0b00_00_01_11;
		let moveX: i8;
		if (state & 0b00_10_00_00) != 0 { moveX = -1; }
		else if (state & 0b00_01_00_00) != 0 { moveX = 1; }
		else { moveX = 0; }
		let jump = (state & 0b00_00_10_00) != 0;
		let attack = (state & 0b10_00_00_00) != 0;
		let protect = (state & 0b01_00_00_00) != 0;
		let px = u16::from_le_bytes([data[1], data[2]]);
		let py = u16::from_le_bytes([data[3], data[4]]);
		let vx = u16::from_le_bytes([data[5], data[6]]);
		let vy = u16::from_le_bytes([data[7], data[8]]);

		return (
			id,
			Self
			{
				pos: (px as f32, py as f32),
				vel: (vx as f32, vy as f32),
				moveX, jump, attack, protect
			}
		);
	}
	fn raw(&self, id: u8) -> Vec<u8>
	{
		let mut state = id;
		if self.moveX == -1 { state = state | 0b00_10_00_00; }
		if self.moveX == 1 { state = state | 0b00_01_00_00; }
		if self.jump { state = state | 0b00_00_10_00; }
		if self.attack { state = state | 0b10_00_00_00; }
		if self.protect { state = state | 0b01_00_00_00; }
		[
			&[state],
			&(self.pos.0.round() as u16).to_le_bytes() as &[u8],
			&(self.pos.1.round() as u16).to_le_bytes() as &[u8],
			&(self.vel.0.round() as u16).to_le_bytes() as &[u8],
			&(self.vel.1.round() as u16).to_le_bytes() as &[u8]
		].concat().to_vec()
	}
}

pub struct Network
{
	tcp: Option<TcpStream>,
	udp: Option<UdpSocket>,
	name: String,
	class: String,
	id: u8,
	tickRate: u8,
	tickTime: Duration,
	mainState: State,
	state: [State; 4]
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
			tickRate: 1,
			tickTime: Duration::from_secs(1),
			mainState: State::default(),
			state: [State::default(); 4]
		}
	}

	pub fn initLua(script: &Lua)
	{
		let table = script.create_table().unwrap();

		table.set("connectTCP", script.create_function(Network::connectTCP).unwrap());
		table.set("receiveTCP", script.create_function(Network::receiveTCP).unwrap());
		table.set("name", script.create_function(Network::name).unwrap());
		table.set("id", script.create_function(Network::id).unwrap());
		table.set("class", script.create_function(Network::class).unwrap());
		table.set("sendTextTCP", script.create_function(Network::sendTextTCP).unwrap());
		table.set("parseID", script.create_function(Network::parseID).unwrap());
		table.set("bytesToU16", script.create_function(Network::bytesToU16).unwrap());
		table.set("parseState", script.create_function(Network::parseState).unwrap());
		table.set("login", script.create_function(Network::login).unwrap());
		table.set("setState", script.create_function(Network::setState).unwrap());
		table.set("getState", script.create_function(Network::getState).unwrap());

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

	pub fn setState(_: &Lua, data: (Number, Number, Number, Number, Table)) -> Result<(), Error>
	{
		let state = &mut Window::getNetwork().mainState;
		state.pos = (data.0 as f32, data.1 as f32);
		state.vel = (data.2 as f32, data.3 as f32);
		state.moveX = data.4.get::<i8>("MoveX").unwrap_or(0);
		state.jump = data.4.get::<bool>("Jump").unwrap_or(false);
		state.attack = data.4.get::<bool>("Attack").unwrap_or(false);
		state.protect = data.4.get::<bool>("Protect").unwrap_or(false);
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

	pub fn name(_: &Lua, _: ()) -> Result<String, Error> { Ok(Window::getNetwork().name.clone()) }
	pub fn id(_: &Lua, _: ()) -> Result<i32, Error> { Ok(Window::getNetwork().id as i32) }
	pub fn class(_: &Lua, _: ()) -> Result<String, Error> { Ok(Window::getNetwork().class.clone()) }

	pub fn parseID(_: &Lua, raw: u8) -> Result<u8, Error>
	{
		Ok((raw & 0b00_00_01_11) + 1)
	}

	pub fn bytesToU16(_: &Lua, raw: (u8, u8)) -> Result<u16, Error>
	{
		Ok(u16::from_le_bytes([raw.0, raw.1]))
	}

	pub fn parseState(_: &Lua, raw: u8) -> Result<(i8, bool, bool, bool), Error>
	{
		let moveX: i8;
		let jump = raw & 0b00_00_10_00 != 0;
		let attack = raw & 0b10_00_00_00 != 0;
		let protect = raw & 0b01_00_00_00 != 0;
		if raw & 0b00_10_00_00 != 0 { moveX = -1; }
		else if raw & 0b00_01_00_00 != 0 { moveX = 1; }
		else { moveX = 0; }
		Ok((moveX, jump, attack, protect))
	}

	pub fn login(_: &Lua, data: Table) -> Result<(), Error>
	{
		let net = Window::getNetwork();

		let mut buffer = Vec::<u8>::new();
		for x in data.pairs::<Value, Integer>()
		{
			if x.is_err() { continue; }
			let (_, b) = x.unwrap();
			buffer.push(b as u8);
		}

		net.id = buffer[0];

		net.name = {
			let mut nameLength = 0;
			while buffer[1 + nameLength] != 0 { nameLength += 1; }
			String::from_utf8_lossy(&buffer[1..1 + nameLength]).to_string()
		};
		
		net.class = {
			let mut classLength = 0;
			while buffer[2 + net.name.len() + classLength] != 0
			{
				classLength += 1;
			}
			String::from_utf8_lossy(
				&buffer[2 + net.name.len()..2 + net.name.len() + classLength]
			).to_string()
		};

		let addr = net.tcp.as_mut().unwrap().peer_addr().unwrap().ip().to_string() + ":" +
		&u16::from_le_bytes([
			buffer[3 + net.name.len() + net.class.len()],
			buffer[4 + net.name.len() + net.class.len()]
		]).to_string();
		println!("Connecting UDP to {addr}");
		net.udp.as_mut().unwrap().connect(addr);

		net.tickRate = buffer[5 + net.name.len() + net.class.len()];

		net.tickTime = Duration::from_secs_f32(1.0 / (net.tickRate as f32));
		
		Window::getWorld().prog.insert(
			String::from("checkpoint"),
			Variable
			{
				num: 0.0,
				string: String::from_utf8_lossy(
					&buffer[6 + net.name.len() + net.class.len()..buffer.len()]
				).to_string()
			}
		);

		std::thread::spawn(Network::networkThread);
		
		Ok(())
	}

	pub fn getState(_: &Lua, id: u8) -> Result<(f32, f32, f32, f32, i8, bool, bool, bool), Error>
	{
		if id == 0
		{
			return Ok((
				0.0, 0.0, 0.0, 0.0,
				0, false, false, false
			))
		}

		let net = Window::getNetwork();
		let s = net.state[(id - 1) as usize];
		Ok((
			s.pos.0, s.pos.1, s.vel.0, s.vel.1,
			s.moveX, s.jump, s.attack, s.protect
		))
	}

	fn receiveUDP(&mut self) -> Option<Vec<u8>>
	{
		let udp = self.udp.as_mut().unwrap();
		let buffer = &mut [0u8; 128];
		let mut result = udp.recv(buffer);
		let mut size = 0;
		while result.is_ok()
		{
			size = result.unwrap();
			result = udp.recv(buffer);
		}
		match result.as_mut().unwrap_err().kind()
		{
			ErrorKind::WouldBlock => {},
			_ =>
			{
				println!("STOPPING NETWORK THREAD; UDP ERROR:\n{}", result.unwrap_err());
				self.udp = None;
				return None;
			}
		}
		Some(buffer[..size].to_vec())
	}

	fn sendUDP(&mut self)
	{
		let udp = self.udp.as_mut().unwrap();
		let data = self.mainState.raw(self.id);
		udp.send(&data);
	}

	pub fn networkThread()
	{
		let net = Window::getNetwork();
		let mut timer = Instant::now();
		'main: loop
		{
			while timer.elapsed() < net.tickTime {}

			let data = net.receiveUDP();
			if data.is_none() { break 'main; }
			let data = data.unwrap();
			if data.len() % 9 != 0
			{
				println!("WRONG UDP PACKET SIZE: {}", data.len());
				net.udp = None;
				break 'main;
			}
			for i in 0..(data.len() / 9)
			{
				let (id, s) = State::parse(&data[i * 9..(i + 1) * 9]);
				net.state[(id - 1) as usize] = s;
			}

			net.sendUDP();
			timer = Instant::now();
		}
	}
}