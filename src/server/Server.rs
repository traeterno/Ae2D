use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::net::{TcpListener, UdpSocket};

use super::WebClient::WebClient;
use super::Transmission::{ClientMessage, ServerMessage};
use super::State::State;
use super::Client::Client;

pub struct Server
{
	clients: HashMap<u8, Client>,
	state: State,
	requests: Vec<(u8, ServerMessage)>,
	broadcast: Vec<ClientMessage>,
	udp: UdpSocket,
	sendTimer: Instant,
	started: bool,
	silent: bool,
	web: WebClient
}

impl Server
{
	pub fn getInstance() -> &'static mut Server
	{
		static mut INSTANCE: Option<Server> = None;
		
		unsafe
		{
			if INSTANCE.is_none() { INSTANCE = Some(Self::init()); }
			INSTANCE.as_mut().expect("Server singleton is not initialized")
		}
	}

	pub fn init() -> Self
	{
		Self
		{
			clients: (||
			{
				let mut c = HashMap::new();
				for i in 1..=5 { c.insert(i, Client::default()); }
				c
			})(),
			state: State::init(),
			requests: vec![],
			broadcast: vec![],
			udp: (||
			{
				match UdpSocket::bind("0.0.0.0:0")
				{
					Ok(s) => { let _ = s.set_nonblocking(true); s},
					Err(x) => panic!("Сокет UDP не создан: {x:?}")
				}
			})(),
			sendTimer: Instant::now(),
			started: false,
			silent: false,
			web: WebClient::init()
		}
	}

	pub fn listen()
	{
		let _ = std::thread::Builder::new().name("Listener".to_string()).spawn(||
		{
			let i = Server::getInstance();
			let bc = UdpSocket::bind("0.0.0.0:26225").unwrap();
			let _ = bc.set_broadcast(true);
			let _ = bc.set_read_timeout(Some(Duration::from_secs_f32(0.1)));
			let l = TcpListener::bind("0.0.0.0:0").unwrap();
			let _ = l.set_nonblocking(true);
			let mut placeholder = [0u8];
			println!("TCP открыт на {}.", l.local_addr().unwrap().port());
			'listener: loop
			{
				if i.started
				{
					println!("Видимость сервера отключена.");
					break 'listener;
				}
				match bc.recv_from(&mut placeholder)
				{
					Ok((_, addr)) =>
					{
						let _ = bc.send_to(
							&l.local_addr().unwrap().port().to_be_bytes() as &[u8],
							addr
						);
					},
					Err(_) => {}
				}
				if let Ok((tcp, addr)) = l.accept()
				{
					let id = i.getAvailablePlayerID();
					if id == 0 { continue; }
					let mut info = i.state.getPlayerInfo(addr.ip());
					if info.name == "noname" { println!("P{id}: Новый игрок."); }
					else { println!("P{id}: {} подключился к игре.", info.name); }

					info.class = Self::updateClass(
						"unknown".to_string(),
						info.class
					);

					i.clients.insert(id, Client::connect(tcp, info.clone()));
					i.broadcast.push(ClientMessage::Login(id, info.name));
					i.checkReady();
					i.updatePlayers();
				}
			}
		});
	}

	pub fn update(&mut self)
	{
		let controller = Instant::now();
		for (id, c) in &mut self.clients
		{
			if c.tcp.is_none() { continue; }
			for req in c.receiveTCP()
			{
				self.requests.push((*id, req));
			}
		}

		self.handleRequests();
		self.broadcastTCP();

		if self.started
		{
			'udp: loop
			{
				let buffer = &mut [0u8; 16];
				match self.udp.recv_from(buffer)
				{
					Ok((size, addr)) =>
					{
						if size != 9 { continue; }
						let id = buffer[0] & 0b00_00_01_11;
						let c = self.clients.get_mut(&id).unwrap();

						if c.udp.is_none() { c.udp = Some(addr); }
						c.state = [buffer[0],
							buffer[1], buffer[2],
							buffer[3], buffer[4],
							buffer[5], buffer[6],
							buffer[7], buffer[8]
						];
					},
					Err(_) => { break 'udp; }
				}
			}
			if self.sendTimer.elapsed() > self.state.settings.sendTime
			{
				self.broadcastState();
				self.sendTimer = Instant::now();
			}
		}

		std::thread::sleep(Duration::from_secs_f32(
			(1.0 / 1000.0 - controller.elapsed().as_secs_f32()).max(0.0)
		));
	}

	fn handleRequests(&mut self)
	{
		for (id, msg) in self.requests.clone()
		{
			match msg
			{
				ServerMessage::Disconnected =>
				{
					println!("P{id} вышел из игры.");
					self.broadcast.push(ClientMessage::Disconnected(id));
				}
				ServerMessage::Chat(msg) =>
				{
					let name = self.clients.get(&id).unwrap().info.name.clone();
					self.newMessage(name, msg);
				}
				ServerMessage::GetGameInfo(kind) =>
				{
					println!("P{id} запросил информацию об игре #{kind}.");
					let mut out = vec![];
					match kind
					{
						0 => // System
						{
							out = [
								&[self.state.settings.tickRate],
								&[self.state.settings.maxItemCellSize],
								&self.udp.local_addr().unwrap().port().to_be_bytes() as &[u8],
								self.state.save.checkpoint.as_bytes(),
								&[0u8]
							].concat();
						}
						x =>
						{
							println!("Неизвестный тип информации: {x}")
						}
					}
					self.clients.get_mut(&id).unwrap().sendTCP(
						ClientMessage::GameInfo(kind, out)
					);
				}
				ServerMessage::GetPlayerInfo(target, kind) =>
				{
					println!("P{id} запросил информацию #{kind} о P{target}.");
					if !self.clients.contains_key(&target) { continue; }
					let acc = self.clients.get(&target).unwrap().info.clone();
					let raw = match kind
					{
						0 => { [acc.name.as_bytes().to_vec(), vec![0u8]].concat() }
						1 => { [acc.class.as_bytes().to_vec(), vec![0u8]].concat() }
						2 => { vec![acc.color.0, acc.color.1, acc.color.2] }
						3 => { acc.hp.to_be_bytes().to_vec() }
						4 =>
						{
							[acc.inv().as_bytes().to_vec(), vec![0u8]].concat()
						}
						x =>
						{
							println!("Неизвестный тип информации: {x}");
							vec![]
						}
					};
					self.clients.get_mut(&id).unwrap().sendTCP(
						ClientMessage::PlayerInfo(target, kind, raw)
					);
				}
				ServerMessage::SetPlayerInfo(kind, raw) =>
				{
					self.setPlayerInfo(id, kind, raw);
				}
				ServerMessage::SetGameInfo(kind, raw) =>
				{
					match kind
					{
						0 =>
						{
							self.started = raw[0] == 1;
							self.broadcast.push(ClientMessage::GameInfo(1, vec![3u8, 0u8]));
							println!("P{id} начал игру.");
						}
						1 =>
						{
							self.state.save(
								String::from_utf8_lossy(&raw).to_string()
							);
							self.broadcast.push(ClientMessage::GameInfo(2, vec![0u8]));
							self.broadcast.push(ClientMessage::Chat(
								String::from("Игра сохранена.")
							));
							println!("P{id} сохранил игру.");
						}
						x =>
						{
							println!("P{id} попытался изменить параметр игры #{x}.");
						}
					}
				}
			}
		}
		self.requests.clear();
	}

	fn broadcastTCP(&mut self)
	{
		let mut check = false;
		for msg in &self.broadcast
		{
			for (_, c) in &mut self.clients
			{
				c.sendTCP(msg.clone());
			}
			match *msg
			{
				ClientMessage::Disconnected(id) =>
				{
					self.clients.insert(id, Client::default());
					check = true;
				}
				_ => {}
			}
		}
		self.broadcast.clear();
		if check
		{
			self.checkReady();
			self.updatePlayers();
		}
	}

	fn broadcastState(&mut self)
	{
		for i in 1..=5
		{
			let addr = self.clients.get(&i).unwrap().udp;
			if addr.is_none() { continue; }
			let addr = addr.unwrap();

			let mut buffer: Vec<u8> = vec![];
			for id in 1..=5
			{
				if self.clients.get(&id).unwrap().udp.is_none() || id == i { continue; }
				buffer.append(&mut self.clients.get(&id).unwrap().state.to_vec());
			}
			if buffer.len() == 0 { continue; }

			let _ = self.udp.send_to(&buffer, addr);
		}
	}
	
	fn getAvailablePlayerID(&self) -> u8
	{
		for id in 1..=10
		{
			if self.clients.get(&id).unwrap().tcp.is_none()
			{
				return id;
			}
		}
		0
	}

	pub fn setStarted(&mut self, started: bool)
	{
		self.started = started;
		if !self.started
		{
			Self::listen();
		}
	}

	pub fn updateClass(previous: String, new: String) -> String
	{
		if previous == new { return String::from("unknown"); }
		for (_, c) in &Self::getInstance().clients
		{
			if c.info.class == new { return previous; }
		}
		new
	}

	pub fn _split(src: String) -> Vec<String>
	{
		let mut v = vec![];

		let mut s = String::new();
		let mut quoted = false;

		for c in src.chars()
		{
			if c == ' '
			{
				if quoted { s.push(c); }
				else if !s.is_empty() { v.push(s); s = String::new(); }
			}
			else if c == '"'
			{
				quoted = !quoted;
				if !s.is_empty() { v.push(s); s = String::new(); }
			}
			else { s.push(c); }
		}

		if !s.is_empty() { v.push(s); }
		
		v
	}

	pub fn getState() -> &'static mut State { &mut Server::getInstance().state }
	pub fn getPlayers() -> &'static HashMap<u8, Client> { &Server::getInstance().clients }

	pub fn checkReady(&mut self)
	{
		let mut ready = true;
		let mut count = 0u8;
		for (_, c) in &self.clients
		{
			if c.tcp.is_none() { continue; }
			count += 1;
			if c.info.name == "noname" || c.info.class == "unknown"
			{
				ready = false;
				break;
			}
		}

		if count == 0
		{
			if self.silent { std::process::exit(0); }
			if self.started
			{
				println!("Все игроки вышли. Возвращаемся в меню...");
				self.setStarted(false);
			}
			return;
		}

		for i in 1..=5
		{
			let c = self.clients.get_mut(&i).unwrap();
			if c.tcp.is_none() { continue; }
			c.sendTCP(ClientMessage::GameInfo(1, vec![
				if ready { 2u8 } else { 1u8 }, 0u8
			]));
			if ready
			{
				println!("P{i} может начать игру.");
			}
			return;
		}
	}

	pub fn setSilent(&mut self, silent: bool)
	{
		self.silent = silent;
	}

	pub fn setPlayerInfo(&mut self, id: u8, kind: u8, mut raw: Vec<u8>)
	{
		let acc = self.clients.get_mut(&id).unwrap();
		let mut check = false;
		match kind
		{
			0 =>
			{
				let name = String::from_utf8_lossy(&raw).to_string();
				println!("P{id} изменил своё имя на \"{name}\".");
				if acc.info.name == "noname"
				{
					acc.sendTCP(
						ClientMessage::Login(id, name.clone())
					);
				}
				acc.info.name = name;
				raw.push(0);
				check = true;
			}
			1 =>
			{
				let class = Self::updateClass(
					acc.info.class.clone(),
					String::from_utf8_lossy(&raw).to_string()
				);
				println!("P{id} изменил свой класс на \"{class}\".");
				raw = class.as_bytes().to_vec();
				raw.push(0);
				acc.info.class = class;
				check = true;
			}
			2 =>
			{
				let r = raw[0];
				let g = raw[1];
				let b = raw[2];
				println!("P{id} изменил свой цвет на ({r}, {g}, {b}).");
				acc.info.color = (r, g, b);
			}
			3 =>
			{
				let hp = u16::from_be_bytes([raw[0], raw[1]]);
				acc.info.hp = hp;
			}
			4 =>
			{
				println!("P{id} попытался изменить свой инвентарь?");
			}
			5 =>
			{
				let (res, item) = acc.info.useItem(raw[0]);
				if res
				{
					println!("P{id} использовал предмет '{item}'.");
				}
				else
				{
					println!("P{id} попытался достать предмет '{item}' из воздуха.");
				}
				acc.sendTCP(ClientMessage::PlayerInfo(
					id, 4, [acc.info.inv().as_bytes(), &[0u8]].concat()
				));
				raw = item.as_bytes().to_vec();
				raw.push(0);
			}
			x =>
			{
				println!("P{id} попытался изменить параметр #{x}.");
			}
		}
		self.broadcast.push(ClientMessage::PlayerInfo(
			id, kind, raw
		));
		self.updatePlayers();
		if check
		{
			self.checkReady();
		}
	}

	pub fn newMessage(&mut self, name: String, mut msg: String)
	{
		let f = msg.chars().nth(0).unwrap();
		if f == '!'
		{
			msg.remove(0);
			println!("{msg}");
			self.broadcast.push(ClientMessage::Chat(msg.clone()));
			self.state.chatHistory.push((name, msg));
		}
		else
		{
			println!("{name}: {msg}");
			self.broadcast.push(ClientMessage::Chat(name.clone() + ": " + &msg));
			self.state.chatHistory.push((name, msg));
		}
		let msg = self.state.jsonChatHistory(
			self.state.chatHistory.len() - 1
		);
		self.web.send("chatMessages", msg);
	}

	pub fn getWC() -> &'static mut WebClient { &mut Self::getInstance().web }

	pub fn updatePlayers(&mut self)
	{
		let players = self.state.jsonPlayers();
		self.web.send("players", players);
	}
}