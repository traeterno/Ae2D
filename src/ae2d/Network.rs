use std::{io::{ErrorKind, Read, Write}, net::{TcpStream, UdpSocket}, time::{Duration, Instant}};

use mlua::{Error, Lua, Table};

use crate::server::Transmission::ClientMessage;

use super::Window::Window;

struct SavedState
{
	checkpoint: String
}

#[derive(Clone, Copy, Debug)]
struct PlayerState
{
	pos: (f32, f32),
	vel: (f32, f32),
	moveX: i8,
	jump: bool,
	attack: bool,
	protect: bool
}

impl PlayerState
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
	mainState: PlayerState,
	state: Vec<PlayerState>,
	save: SavedState,
	tcpHistory: Vec<ClientMessage>
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
			mainState: PlayerState::default(),
			state: vec![],
			save: SavedState { checkpoint: String::new() },
			tcpHistory: vec![]
		}
	}

	pub fn initLua(script: &Lua)
	{
		let table = script.create_table().unwrap();

		table.set("name", script.create_function(Network::name).unwrap());
		table.set("id", script.create_function(Network::id).unwrap());
		table.set("class", script.create_function(Network::class).unwrap());
		table.set("connect", script.create_function(Network::connect).unwrap());
		table.set("login", script.create_function(Network::login).unwrap());
		table.set("getState", script.create_function(Network::getState).unwrap());
		table.set("hasMessage", script.create_function(Network::hasMessage).unwrap());
		table.set("getMessage", script.create_function(Network::getMessage).unwrap());
		table.set("sendMessage", script.create_function(Network::sendMessage).unwrap());

		script.globals().set("network", table);
	}

	fn name(_: &Lua, _: ()) -> Result<String, Error> { Ok(Window::getNetwork().name.clone()) }
	fn id(_: &Lua, _: ()) -> Result<u8, Error> { Ok(Window::getNetwork().id) }
	fn class(_: &Lua, _: ()) -> Result<String, Error> { Ok(Window::getNetwork().class.clone()) }

	fn connect(_: &Lua, addr: String) -> Result<bool, Error>
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
		std::thread::spawn(Network::tcpThread);
		Ok(true)
	}

	fn login(_: &Lua, data: (u8, String, String)) -> Result<(), Error>
	{
		let net = Window::getNetwork();

		net.id = data.0;
		net.name = data.1;
		net.class = data.2;

		// let addr = net.tcp.as_mut().unwrap().peer_addr().unwrap().ip().to_string() + ":" +
		// &u16::from_le_bytes([
		// 	buffer[3 + net.name.len() + net.class.len()],
		// 	buffer[4 + net.name.len() + net.class.len()]
		// ]).to_string();
		// println!("Connecting UDP to {addr}");
		// net.udp.as_mut().unwrap().connect(addr);

		// net.tickRate = buffer[5 + net.name.len() + net.class.len()];

		// net.state.resize(
		// 	buffer[6 + net.name.len() + net.class.len()] as usize,
		// 	PlayerState::default()
		// );

		// net.tickTime = Duration::from_secs_f32(1.0 / (net.tickRate as f32));
		
		// net.save.checkpoint = String::from_utf8_lossy(
		// 	&buffer[7 + net.name.len() + net.class.len()..buffer.len()]
		// ).to_string();

		std::thread::spawn(Network::updateThread);
		
		Ok(())
	}

	fn getState(_: &Lua, id: u8) -> Result<(f32, f32, f32, f32, i8, bool, bool, bool), Error>
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

	fn hasMessage(_: &Lua, id: u8) -> Result<bool, Error>
	{
		for msg in &Window::getNetwork().tcpHistory
		{
			match msg
			{
				ClientMessage::Login(..) => if id == 1 { return Ok(true); }
				ClientMessage::Disconnected(..) => if id == 2 { return Ok(true); }
				ClientMessage::Chat(..) => if id == 3 { return Ok(true); }
				ClientMessage::SetPosition(..) => if id == 4 { return Ok(true); }
				ClientMessage::GetInfo(..) => if id == 5 { return Ok(true); }
			}
		}
		Ok(false)
	}

	fn getMessage(script: &Lua, msg: u8) -> Result<Table, Error>
	{
		let table = script.create_table().unwrap();

		let net = Window::getNetwork();

		for i in 0..net.tcpHistory.len()
		{
			let mut found = false;
			match &net.tcpHistory[i]
			{
				ClientMessage::Login(id, name, class) =>
				{
					if msg != 1 { continue; }
					table.raw_set("id", *id);
					table.raw_set("name", name.clone());
					table.raw_set("class", class.clone());
					found = true;
				},
				_ => {}
			}
			if found { net.tcpHistory.swap_remove(i); break; }
		}

		Ok(table)
	}

	fn sendMessage(_: &Lua, (msg, data): (u8, Table)) -> Result<(), Error>
	{
		let net = Window::getNetwork();
		let tcp = net.tcp.as_mut().unwrap();
		tcp.write(&match msg
		{
			1 =>
			{
				let name: String = data.get("name").unwrap();
				[&[1u8], name.as_bytes()].concat()
			}
			_ => vec![]
		});
		Ok(())
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
		if size == 0 { return Some(vec![]); }
		Some(buffer[..size].to_vec())
	}

	pub fn updateThread()
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
				let (id, s) = PlayerState::parse(&data[i * 9..(i + 1) * 9]);
				net.state[(id - 1) as usize] = s;
			}

			let udp = net.udp.as_mut().unwrap();
			udp.send(&net.mainState.raw(net.id));

			timer = Instant::now();
		}
	}

	pub fn tcpThread()
	{
		let net = Window::getNetwork();
		let tcp = net.tcp.as_mut().unwrap();
		let buf = &mut [0u8; 256];
		'main: loop
		{
			match tcp.read(buf)
			{
				Ok(size) =>
				{
					if let Some(msg) = Network::parse(&buf[0..size])
					{
						net.tcpHistory.push(msg);
					}
				},
				Err(x) =>
				{
					match x.kind()
					{
						ErrorKind::WouldBlock => {},
						ErrorKind::ConnectionRefused =>
						{
							Window::getNetwork().tcp = None;
							break 'main;
						},
						_ => {}
					}
				}
			}
		}
	}

	fn parse(buffer: &[u8]) -> Option<ClientMessage>
	{
		return match buffer[0]
		{
			1 =>
			{
				let id = buffer[1];

				let name =
				{
					let mut len = 0;
					while buffer[2 + len] != 0 { len += 1; }
					String::from_utf8_lossy(
						&buffer[2..2 + len]
					).to_string()
				};

				let class = String::from_utf8_lossy(
					&buffer[3 + name.len()..buffer.len()]
				).to_string();
				
				Some(ClientMessage::Login(id, name, class))
			},
			_ => None
		}
	}
}