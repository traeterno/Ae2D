use std::time::Instant;
use std::net::{SocketAddr, TcpListener, UdpSocket};

use super::WebClient::WebClient;
use super::Transmission::{ClientMessage, ServerMessage, WebResponse};
use super::State::State;
use super::Config::{Config, Permission};
use super::Client::Client;

pub struct Server
{
	listener: TcpListener,
	webListener: TcpListener,
	webClient: WebClient,
	clients: Vec<Client>,
	config: Config,
	state: State,
	requests: Vec<(u8, ServerMessage)>,
	broadcast: Vec<ClientMessage>,
	udp: UdpSocket,
	playersState: Vec<[u8; 9]>,
	sendTimer: Instant,
	recvTimer: Instant,
	udpBC: UdpSocket,
	visible: bool
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
		let config = Config::init();
		let state = State::init(&config);

		let listener = TcpListener::bind(String::from("0.0.0.0:") + &config.port.to_string());
		if listener.is_err() { panic!("Failed to create listener: {:?}", listener.unwrap_err()); }
		let listener = listener.unwrap();
		let _ = listener.set_nonblocking(true);

		let webListener = TcpListener::bind("0.0.0.0:8080");
		if webListener.is_err() { panic!("Failed to create web listener: {:?}", webListener.unwrap_err()); }
		let webListener = webListener.unwrap();
		let _ = webListener.set_nonblocking(true);

		let mut clients = vec![];
		clients.resize_with(config.getPlayersCount() as usize, || { Client::default() });

		let mut playersState = vec![];
		playersState.resize(config.getPlayersCount() as usize, [0u8; 9]);

		let udp = UdpSocket::bind("0.0.0.0:0");
		if udp.is_err()
		{
			panic!("Failed to bind UDP: {:?}", udp.unwrap_err());
		}
		let udp = udp.unwrap();
		let _ = udp.set_nonblocking(true);

		let bc = UdpSocket::bind("0.0.0.0:26225");
		if bc.is_err()
		{
			panic!("Failed to bind UDP Broadcast: {:?}", bc.unwrap_err());
		}
		let bc = bc.unwrap();
		let _ = bc.set_nonblocking(true);
		let _ = bc.set_broadcast(true);

		Self
		{
			listener,
			webListener,
			webClient: WebClient::new(),
			clients,
			config,
			state,
			requests: vec![],
			broadcast: vec![],
			udp,
			playersState,
			sendTimer: Instant::now(),
			recvTimer: Instant::now(),
			udpBC: bc,
			visible: false
		}
	}

	pub fn listen(&mut self)
	{
		if let Ok((tcp, addr)) = self.listener.accept()
		{
			let id = self.getAvailablePlayerID();
			if id != 0
			{
				let (name, class) = self.state.getPlayerInfo(addr.ip());
				if name == "noname" { println!("Новый игрок."); }
				else { println!("Игрок {name} подключился, как P{}.", id); }

				self.clients[(id - 1) as usize] = Client::connect(
					tcp,
					id,
					name.clone(),
					class.clone()
				);

				self.broadcast.push(ClientMessage::Login(id, name, class))
			}
		}

		for client in self.webListener.incoming()
		{
			match client
			{
				Ok(tcp) => self.webClient.connect(tcp),
				Err(_) => break
			}
		}

		if self.udpBC.broadcast().unwrap()
		{
			let mut buf = [0u8; 64];
			match self.udpBC.recv_from(&mut buf)
			{
				Ok((_, addr)) =>
				{
					if !self.visible { return; }
					let _ = self.udpBC.send_to(
						&self.listener.local_addr().unwrap().port().to_le_bytes() as &[u8],
						addr
					);
				}
				Err(x) =>
				{
					match x.kind()
					{
						std::io::ErrorKind::WouldBlock => {}
						_ => println!("Error occured: {x:?}")
					}
				}
			}
		}
	}

	pub fn update(&mut self)
	{
		if self.recvTimer.elapsed() > self.config.recvTime
		{
			for msg in self.webClient.update()
			{
				self.requests.push((0, msg));
			}
	
			for c in &mut self.clients
			{
				if c.tcp.is_none() { continue; }
				if let Some(req) = c.receiveTCP()
				{
					self.requests.push((c.id, req));
				}
			}
	
			'udp: loop
			{
				let buffer = &mut [0u8; 128];
				match self.udp.recv_from(buffer)
				{
					Ok((size, addr)) =>
					{
						if size != 9 { continue; }
						let id = buffer[0] & 0b00_00_01_11;
						if self.clients[(id - 1) as usize].udp.is_none()
						{
							self.clients[(id - 1) as usize].udp = Some(addr);
						}
						self.playersState[(id - 1) as usize] = [buffer[0],
							buffer[1], buffer[2],
							buffer[3], buffer[4],
							buffer[5], buffer[6],
							buffer[7], buffer[8]
						];
					},
					Err(_) => { break 'udp; }
				}
			}
			self.recvTimer = Instant::now();
		}
		
		self.handleRequests();
		self.broadcastTCP();

		if self.sendTimer.elapsed() > self.config.sendTime
		{
			self.broadcastState();
			self.sendTimer = Instant::now();
		}
	}

	fn handleRequests(&mut self)
	{
		for (id, msg) in self.requests.clone()
		{
			match msg
			{
				ServerMessage::Invalid(web) =>
				{
					if id == 0
					{
						WebClient::sendResponse(web, WebResponse::Ok(
							String::from("{ \"error\": \"Invalid or unknown request\" }"),
							String::from("text/json")
						));
					}
				},
				ServerMessage::Register(name) =>
				{
					let c = &mut self.clients[(id - 1) as usize];
					c.name = name.clone();

					self.broadcast.push(ClientMessage::Login(
						id, name.clone(), String::from("unknown"),
					));

					self.state.setPlayerInfo(
						c.tcp.as_mut().unwrap().peer_addr().unwrap().ip(),
						name.clone(), String::from("unknown")
					);

					self.config.setPermission(name.clone(), Permission::Player);

					println!("Welcome, {name}(P{id})!");
				},
				ServerMessage::Disconnected =>
				{
					if id != 0
					{
						println!("P{} disconnected.", id);
						self.clients[(id - 1) as usize] = Client::default();
						self.playersState[(id - 1) as usize][0] = id;
						self.broadcast.push(ClientMessage::Disconnected(id));
					}
				},
				ServerMessage::Chat(msg, web) =>
				{
					println!("P{id}: {msg}");
					let mut text = msg.clone();
					let c = text.remove(0);
					if c == '/' { self.cmd(id, web, text); }
					else
					{
						let n =
							if id == 0 { String::from("WebClient") }
							else { self.clients[(id - 1) as usize].name.clone() };
						self.broadcast.push(ClientMessage::Chat(n.clone() + ": " + &msg));
						self.state.chatHistory.push((n.clone(), msg.clone()));
						if id == 0
						{
							WebClient::sendResponse(web, WebResponse::Ok(
								String::from("{ \"msg\": \"") + &n + ": " + &msg + "\" }",
								"text/json".to_string()
							));
						}
					}
				},
				ServerMessage::PlayersList(web) =>
				{
					let mut obj = json::JsonValue::new_array();

					for c in &self.clients
					{
						if c.id == 0 { continue; }
						
						let class = match c.class.as_str()
						{
							"sorcerer" => "Маг",
							"thief" => "Вор",
							"knight" => "Рыцарь",
							"engineer" => "Инженер",
							"bard" => "Бард",
							_ => "Неизвестный"
						};

						let _ = obj.push(json::object!
						{
							id: c.id,
							className: class,
							name: c.name.clone(),
							hp: { current: 100, max: 100 },
							mana: { current: 100, max: 100 }
						});
					}

					WebClient::sendResponse(web, WebResponse::Ok(
						json::stringify(obj), "text/json".to_string()
					));
				},
				ServerMessage::SaveGame(checkpoint) =>
				{
					println!("Game saved on {checkpoint}.");
					self.save(checkpoint);
				},
				ServerMessage::ChatHistory(mut start, web) =>
				{
					if start > self.state.chatHistory.len() { start = 0; }
					let count = self.state.chatHistory.len() - start;
					let mut buf = json::JsonValue::new_array();
					for i in start..self.state.chatHistory.len()
					{
						let (user, msg) = &self.state.chatHistory[
							if count > 1 { self.state.chatHistory.len() - 1 - i }
							else { i }
						];
						let mut obj = json::JsonValue::new_object();
						let _ = obj.insert("user", user.clone());
						let _ = obj.insert("msg", msg.clone());
						let _ = buf.push(obj);
					}
					WebClient::sendResponse(web, WebResponse::Ok(
						json::stringify(buf), "text/json".to_string()
					));
				},
				ServerMessage::GameState(web) =>
				{
					let mut msg = json::JsonValue::new_array();

					let _ = msg.push(json::object!
					{
						title: "Сохранение",
						props: json::object!
						{
							"Чекпоинт": self.state.lastCheckpoint.as_str(),
							"Дата сохранения": self.state.date.as_str()
						}
					});

					WebClient::sendResponse(web, WebResponse::Ok(
						json::stringify(msg), "text/json".to_string()
					));
				},
				ServerMessage::ChatLength(web) =>
				{
					WebClient::sendResponse(web, WebResponse::Ok(
						self.state.chatHistory.len().to_string(), "text/json".to_string()
					));
				},
				ServerMessage::GetSettings(web) =>
				{
					let mut msg = json::JsonValue::new_object();

					let _ = msg.insert("Сервер", json::object!
					{
						extendedPlayers: json::object!
						{
							type: "toggle",
							name: "Расширить количество игроков",
							value: self.config.extendedPlayers
						},
						port: json::object!
						{
							type: "range",
							name: "Игровой порт",
							value: self.config.port,
							props: json::object! { min: 1024, max: u16::MAX }
						},
						tickRate: json::object!
						{
							type: "range",
							name: "Частота обновления",
							value: self.config.tickRate,
							props: json::object! { min: 1, max: 100 }
						},
						firstCP: json::object!
						{
							type: "string",
							name: "Начальный чекпоинт",
							value: self.config.firstCheckpoint.clone(),
						}
					});

					let mut perms = json::JsonValue::new_object();
					
					for (name, group) in &self.config.permissions
					{
						let p = match group
						{
							Permission::Player => "Игрок",
							Permission::Developer => "Разработчик"
						};
						let _ = perms.insert(&name, json::object!
						{
							type: "list",
							name: name.clone(),
							value: p,
							props: json::array![ "Игрок", "Разработчик" ]
						});
					}

					let _ = msg.insert("Разрешения игроков", perms);

					WebClient::sendResponse(web, WebResponse::Ok(
						json::stringify(msg), "text/json".to_string()
					));
				},
				ServerMessage::SaveSettings(web) =>
				{
					println!("Настройки сервера были изменены.");
					WebClient::sendResponse(web, WebResponse::Ok(
						"{}".to_string(), "text/json".to_string()
					));
				},
				ServerMessage::GetInfo =>
				{
					let mut players = vec![];
					
					for p in &self.clients
					{
						if p.id == 0 { break; }
						players.push(p.id.to_string() + "/" + &p.name + "/" + &p.class);
					}

					self.clients[(id - 1) as usize].sendTCP(ClientMessage::GetInfo(
						self.udp.local_addr().unwrap().port(),
						self.config.tickRate,
						self.state.checkpoints.clone(),
						self.config.extendedPlayers, players
					));
				},
				ServerMessage::SelectChar(avatar) =>
				{
					let avatar = (avatar - 1) % 5;
					let mut class = match avatar
					{
						0 => "sorcerer",
						1 => "thief",
						2 => "knight",
						3 => "engineer",
						4 => "bard",
						_ => "unknown"
					};
					let c = &mut self.clients[(id - 1) as usize];
					if c.class == class { class = "unknown"; }
					c.class = class.to_string();
					self.state.playersList.get_mut(
						&c.tcp.as_mut().unwrap().peer_addr().unwrap().ip()
					).unwrap().1 = c.class.clone();
					self.broadcast.push(ClientMessage::SelectChar(id, class.to_string()));
				}
			}
		}
		self.requests.clear();
	}

	fn broadcastTCP(&mut self)
	{
		for msg in &self.broadcast
		{
			for c in &mut self.clients
			{
				c.sendTCP(msg.clone());
			}
		}
		self.broadcast.clear();
	}

	fn broadcastState(&mut self)
	{
		for i in 0..self.config.getPlayersCount() as usize
		{
			if i >= self.clients.len() { break; }
			let addr = self.clients[i].udp;
			if addr.is_none() { continue; }
			let addr = addr.unwrap();

			let mut buffer: Vec<u8> = vec![];
			for id in 0..self.config.getPlayersCount() as usize
			{
				if self.playersState[id][0] == 0 || id == i { continue; }
				buffer.append(&mut self.playersState[id].to_vec());
			}
			if buffer.len() == 0 { continue; }

			let _ = self.udp.send_to(&buffer, addr);
		}
	}

	fn save(&mut self, checkpoint: String)
	{
		self.config.save();
		self.state.save(checkpoint);
	}
	
	fn getAvailablePlayerID(&self) -> u8
	{
		for i in 0..self.config.getPlayersCount() as usize
		{
			if self.clients[i].id == 0 { return (i + 1) as u8; }
		}
		0
	}

	fn getPlayerID(&self, name: &str) -> u8
	{
		for i in 0..self.config.getPlayersCount() as usize
		{
			if self.clients[i].name.to_lowercase() == name.to_lowercase()
			{
				return (i + 1) as u8;
			}
		}
		0
	}

	pub fn cmd(&mut self, executor: u8, webID: SocketAddr, txt: String)
	{
		let txt = txt.to_lowercase();
		let mut args = txt.split(" ");
		if executor == 0
		{
			println!("Центр мира вызвал команду: {txt}");
			WebClient::sendResponse(webID, WebResponse::Ok(
				String::from("{ \"msg\": \"") + &txt + "\" }",
				"text/json".to_string()
			));
		}
		let name =
			if executor == 0 { &String::from("Центр мира") }
			else { &self.clients[(executor - 1) as usize].name };
		let p = self.config.getPermission(&name);
		println!("P{executor} ({name}, {}) вызвал '{txt}'", p.toString());
		
		let c = args.nth(0).unwrap_or(" ");

		if c == "getposition" && p.check(Permission::Developer)
		{
			let n = args.nth(0).unwrap_or(&name);
			let id = self.getPlayerID(n);

			let pos = if id == 0 { "Не найден" } else
			{
				let s = &self.playersState[(id - 1) as usize];
				let x = u16::from_le_bytes([s[1], s[2]]);
				let y = u16::from_le_bytes([s[3], s[4]]);
				&(x.to_string() + " " + &y.to_string())
			};
			
			let msg = format!("[Игрок {name} запросил координаты {n}] {pos}");

			self.broadcast.push(ClientMessage::Chat(msg.clone()));
			self.state.chatHistory.push((name.to_string(), msg));
		}
		else if c == "setposition" && p.check(Permission::Developer)
		{
			let n = args.nth(0).unwrap_or(&name);
			let id = self.getPlayerID(n);
			if id == 0
			{
				self.state.chatHistory.push((name.clone(),
					format!("[Игрок {n} не был перемещён: НЕ НАЙДЕН]")
				));
				return;
			}
			let x = args.nth(0).unwrap_or("0").parse::<u16>().unwrap();
			let y = args.nth(0).unwrap_or("0").parse::<u16>().unwrap();
			println!("P{id}({n}) перемещён в ({x};{y})");
			
			self.state.chatHistory.push((name.clone(),
				format!("[Игрок {n} перемещён в ({x};{y})]")
			));
			self.clients[(id - 1) as usize].sendTCP(ClientMessage::SetPosition(x, y));
		}
		else if c == "gettime"
		{
			self.state.chatHistory.push((name.clone(),
				format!("Текущее время сервера: {}", State::getDateTime())
			));
		}
		else if c == "save"
		{
			self.save(
				args.nth(0).unwrap_or(&self.config.firstCheckpoint)
				.to_string()
			);
		}
	}

	pub fn setVisible(&mut self, visible: bool)
	{
		self.visible = visible;
	}

	pub fn getWebClient(&mut self) -> &mut WebClient { &mut self.webClient }
	pub fn getConfig(&mut self) -> &mut Config { &mut self.config }
}