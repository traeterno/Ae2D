use std::{collections::HashMap, io::{ErrorKind, Read}, net::{TcpStream, UdpSocket}, time::{Duration, Instant}};

use crate::server::{State::Account, Transmission::ClientMessage};

use super::Window::Window;

#[derive(Clone, Copy, Debug)]
pub struct PlayerState
{
	pub pos: (f32, f32),
	pub vel: (f32, f32),
	pub moveX: i8,
	pub jump: bool,
	pub attack: bool,
	pub protect: bool,
	pub updated: bool
}

impl Default for PlayerState
{
	fn default() -> Self
	{
		Self
		{
			pos: (0.0, 0.0), vel: (0.0, 0.0),
			moveX: 0, jump: false, attack: false, protect: false,
			updated: false
		}
	}
}

impl PlayerState
{
	fn parse(data: &[u8]) -> (u8, Self)
	{
		let state = data[0];
		let moveX: i8;
		if (state & 0b00_10_00_00) != 0 { moveX = -1; }
		else if (state & 0b00_01_00_00) != 0 { moveX = 1; }
		else { moveX = 0; }

		return (
			state & 0b00_00_01_11,
			Self
			{
				pos: (
					u16::from_be_bytes([data[1], data[2]]) as f32,
					u16::from_be_bytes([data[3], data[4]]) as f32
				),
				vel: (
					u16::from_be_bytes([data[5], data[6]]) as f32,
					u16::from_be_bytes([data[7], data[8]]) as f32
				),
				moveX,
				jump: (state & 0b00_00_10_00) != 0,
				attack: (state & 0b10_00_00_00) != 0,
				protect: (state & 0b01_00_00_00) != 0,
				updated: true
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
			&(self.pos.0.round() as u16).to_be_bytes() as &[u8],
			&(self.pos.1.round() as u16).to_be_bytes() as &[u8],
			&(self.vel.0.round() as u16).to_be_bytes() as &[u8],
			&(self.vel.1.round() as u16).to_be_bytes() as &[u8]
		].concat().to_vec()
	}
}

pub struct Network
{
	pub tcp: Option<TcpStream>,
	pub udp: Option<UdpSocket>,
	pub id: u8,
	tickRate: u8,
	tickTime: Duration,
	mainState: PlayerState,
	pub state: Vec<PlayerState>,
	pub tcpHistory: Vec<ClientMessage>,
	pub avatars: HashMap<u8, Account>
}

impl Network
{
	pub fn new() -> Self
	{
		Self
		{
			tcp: None,
			udp: None,
			id: 0,
			tickRate: 1,
			tickTime: Duration::from_secs(1),
			mainState: PlayerState::default(),
			state: vec![],
			tcpHistory: vec![],
			avatars: HashMap::new()
		}
	}

	pub fn setup(&mut self, udp: u16, tickRate: u8, players: mlua::Table)
	{
		let addr = self.tcp.as_mut().unwrap().peer_addr().unwrap().ip()
			.to_string() + ":" + &udp.to_string();
		
		match self.udp.as_mut().unwrap().connect(addr)
		{
			Ok(_) => {}
			Err(x) => println!("Failed: {x}")
		}

		self.tickRate = tickRate;
		self.tickTime = Duration::from_secs_f32(1.0 / self.tickRate as f32);

		for entry in players.pairs::<i32, mlua::Table>()
		{
			if let Ok((id, data)) = entry
			{
				let name: String = data.raw_get("name").unwrap();
				if name == "noname"
				{
					self.avatars.remove(&(id as u8));
					continue;
				}
				let c: mlua::Table = data.raw_get("color").unwrap();
				self.avatars.insert(id as u8, Account
				{
					name: data.raw_get("name").unwrap(),
					class: data.raw_get("class").unwrap(),
					color: (
						c.raw_get("r").unwrap(),
						c.raw_get("g").unwrap(),
						c.raw_get("b").unwrap()
					),
					hp: 100
				});
			}
		}

		std::thread::spawn(Network::updateThread);
	}

	pub fn setPlayersCount(&mut self, playersCount: u8)
	{
		for i in 1..=playersCount
		{
			self.avatars.insert(i, Account::default());
		}
		self.state.resize(playersCount as usize, PlayerState::default());
	}

	pub fn getEP(&self) -> bool
	{
		self.state.len() / 5 == 2
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
		// TODO try udp.set_nonblocking(false) and optimize the code to get lower cpu usage
		// (current going 2% -> 27%)
		let net = Window::getNetwork();
		let mut timer = Instant::now();
		'main: loop
		{
			let controller = Instant::now();
			let data = net.receiveUDP();
			if data.is_none() { break 'main; }
			let data = data.unwrap();
			if data.len() > 0
			{
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
			}

			if timer.elapsed() > net.tickTime
			{
				let udp = net.udp.as_mut().unwrap();
				let _ = udp.send(&net.mainState.raw(net.id));
				timer = Instant::now();
			}

			std::thread::sleep(Duration::from_secs_f32(
				((1.0 / (net.tickRate * 2) as f32) - controller.elapsed().as_secs_f32())
				.max(0.0)
			));
		}
	}

	pub fn tcpThread()
	{
		let net = Window::getNetwork();
		let buf = &mut [0u8; 256];
		'main: loop
		{
			if let Some(tcp) = net.tcp.as_mut()
			{
				match tcp.read(buf)
				{
					Ok(size) =>
					{
						net.tcpHistory.append(&mut Network::parse(&buf[0..size]));
					},
					Err(x) =>
					{
						match x.kind()
						{
							ErrorKind::WouldBlock => {},
							ErrorKind::ConnectionRefused =>
							{
								Window::getNetwork().tcp = None;
								net.tcpHistory.push(ClientMessage::Disconnected(net.id));
								break 'main;
							},
							_ => {}
						}
					}
				}
			}
			else { break 'main; }
		}
	}

	fn parse(buffer: &[u8]) -> Vec<ClientMessage>
	{
		let mut out = vec![];
		let mut current = 0;
		while current < buffer.len()
		{
			match buffer[current]
			{
				1 =>
				{
					let id = buffer[current + 1];
					let name =
					{
						let mut len = 0;
						while buffer[current + 2 + len] != 0 { len += 1; }
						String::from_utf8_lossy(
							&buffer[current + 2..current + 2 + len]
						).to_string()
					};
					current += 3 + name.len();
					out.push(ClientMessage::Login(id, name));
				}
				2 =>
				{
					let id = buffer[current + 1];
					current += 2;
					out.push(ClientMessage::Disconnected(id));
				}
				3 =>
				{
					let msg =
					{
						let mut len = 0;
						while buffer[current + 1 + len] != 0 { len += 1; }
						String::from_utf8_lossy(&buffer[
							current + 1..
							current + 1 + len
						]).to_string()
					};
					current += 2 + msg.len();
					out.push(ClientMessage::Chat(msg));
				}
				4 =>
				{
					let raw = {
						let mut len = 0;
						while buffer[current + 2 + len] != 0 { len += 1; }
						&buffer[current + 2..current + 2 + len]
					};
					out.push(ClientMessage::GameInfo(
						buffer[current + 1],
						raw.to_vec()
					));
					current += 3 + raw.len();
				}
				5 =>
				{
					let id = buffer[current + 1];
					let kind = buffer[current + 2];
					let raw = match kind
					{
						0 =>
						{
							let mut len = 0;
							while buffer[current + 3 + len] != 0 { len += 1; }
							let v = buffer[current + 3..current + 3 + len].to_vec();
							current += 4 + len; v
						}
						1 =>
						{
							let mut len = 0;
							while buffer[current + 3 + len] != 0 { len += 1; }
							let v = buffer[current + 3..current + 3 + len].to_vec();
							current += 4 + len; v
						}
						2 =>
						{
							let v = buffer[current + 3..current + 6].to_vec();
							current += 6; v
						}
						3 =>
						{
							let v = buffer[current + 3..current + 5].to_vec();
							current += 5; v
						}
						x => { println!("Unknown info: {x}"); vec![] }
					};
					out.push(ClientMessage::PlayerInfo(id, kind, raw));
				}
				x =>
				{
					println!("Unknown byte: {x}");
					current += 1;
				}
			}
		}
		out
	}

	pub fn setState(&mut self, s: PlayerState)
	{
		self.mainState = s;
	}
}