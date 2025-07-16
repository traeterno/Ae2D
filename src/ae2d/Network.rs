use std::{io::{ErrorKind, Read}, net::{TcpStream, UdpSocket}, time::{Duration, Instant}};

use crate::server::Transmission::ClientMessage;

use super::Window::Window;

#[derive(Clone, Copy, Debug)]
pub struct PlayerState
{
	pub pos: (f32, f32),
	pub vel: (f32, f32),
	pub moveX: i8,
	pub jump: bool,
	pub attack: bool,
	pub protect: bool
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
	pub tcp: Option<TcpStream>,
	pub udp: Option<UdpSocket>,
	pub name: String,
	pub class: String,
	pub id: u8,
	tickRate: u8,
	tickTime: Duration,
	mainState: PlayerState,
	pub state: Vec<PlayerState>,
	pub tcpHistory: Vec<ClientMessage>
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
			tcpHistory: vec![]
		}
	}

	pub fn setup(&mut self, udp: u16, tickRate: u8)
	{
		let addr = self.tcp.as_mut().unwrap().peer_addr().unwrap().ip()
			.to_string() + ":" + &udp.to_string();
		println!("Connecting UDP to {addr}");
		
		match self.udp.as_mut().unwrap().connect(addr)
		{
			Ok(_) => {}
			Err(x) => println!("Failed: {x}")
		}

		self.tickRate = tickRate;
		self.tickTime = Duration::from_secs_f32(1.0 / self.tickRate as f32);

		std::thread::spawn(Network::updateThread);
	}

	pub fn setEP(&mut self, extendPlayers: bool)
	{
		self.state.resize(
			5 * if extendPlayers { 2 } else { 1 },
			PlayerState::default()
		);
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
			let _ = udp.send(&net.mainState.raw(net.id));

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
						println!("New message: {msg:?}");
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
			2 =>
			{
				Some(ClientMessage::Disconnected(buffer[1]))
			}
			5 =>
			{
				let udpPort = u16::from_le_bytes([buffer[1], buffer[2]]);
				let tickRate = buffer[3];
				let extendPlayers = buffer[4] != 0;
				
				let players =
				{
					let mut v = vec![];
					let mut len = 0;
					while buffer[5 + len] != 0 { len += 1; }
					len -= 1;
					let raw = String::from_utf8_lossy(&buffer[5..5 + len]).to_string();
					for p in raw.split("|") { v.push(p.to_string()) }
					v
				};

				let pl =
				{
					let mut size = 0;
					for p in &players { size += p.len(); }
					size
				};

				let checkpoints =
				{
					let mut v = vec![];
					let raw = String::from_utf8_lossy(
						&buffer[7 + pl..buffer.len() - 1])
						.to_string();
					for c in raw.split("|")
					{
						v.push(c.to_string());
					}
					v
				};
				
				Some(ClientMessage::GetInfo(
					udpPort, tickRate, checkpoints, extendPlayers, players
				))
			},
			6 =>
			{
				Some(ClientMessage::SelectChar(
					buffer[1],
					String::from_utf8_lossy(&buffer[2..buffer.len()]).to_string()
				))
			}
			_ => None
		}
	}
}