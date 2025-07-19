use std::{io::{ErrorKind, Read, Write}, net::{SocketAddr, TcpStream}, time::Duration};

use crate::server::Server::Server;

use super::Transmission::{ServerMessage, WebRequest, WebResponse};

pub struct WebClient
{
	pub tcp: Vec<TcpStream>
}

impl WebClient
{
	pub fn new() -> Self
	{
		Self { tcp: vec![] }
	}
	
	pub fn connect(&mut self, tcp: TcpStream)
	{
		self.tcp.push(tcp);
	}

	pub fn update(&mut self) -> Vec<ServerMessage>
	{
		let mut req = vec![];
		for i in 0..self.tcp.len()
		{
			if i >= self.tcp.len() { break; }
			let buffer = &mut [0u8; 1024];
			if self.tcp[i].peer_addr().is_err()
			{
				self.tcp.swap_remove(i);
			}
			let addr = self.tcp[i].peer_addr().unwrap();
			match self.tcp[i].read(buffer)
			{
				Ok(size) =>
				{
					if size == 0 { continue; }
					let msg = String::from_utf8_lossy(&buffer[0..size]).to_string();
					match WebRequest::build(msg)
					{
						WebRequest::Invalid => continue,
						WebRequest::Get(data) => WebClient::get(addr, data),
						WebRequest::Post(data) => req.push(WebClient::post(addr, data))
					}
				},
				Err(_) => {}
			}
		}

		req
	}

	fn get(id: SocketAddr, data: String)
	{
		let data = data.split("?").collect::<Vec<&str>>()[0];
		if data == "/"
		{
			WebClient::sendResponse(id,
				WebResponse::MovedPermanently(String::from("/index.html")),
			);
		}
		else
		{
			let path = String::from("res/web") + &data;
			WebClient::sendResponse(id,
				match std::fs::read_to_string(path.clone())
				{
					Ok(text) =>
					{
						WebResponse::Ok(text, match path.split(".").last().unwrap()
						{
							"js" => String::from("text/javascript"),
							s => String::from("text/") + s
						})
					},
					Err(x) => match x.kind()
					{
						ErrorKind::InvalidData => match std::fs::read(path.clone())
						{
							Ok(data) =>
							{
								WebResponse::OkRaw(data, match path.split(".").last().unwrap()
								{
									"png" => String::from("image/png"),
									"otf" => String::from("application/x-font-opentype"),
									s => { println!("Unknown file: {s}"); String::from(s) }
								})
							},
							Err(x) => { println!("{x:#?}"); WebResponse::NotFound }
						},
						_ => { println!("{x:#?}"); WebResponse::NotFound }
					}
				}
			);
		}
	}

	fn post(id: SocketAddr, data: String) -> ServerMessage
	{
		match json::parse(&data)
		{
			Ok(parsed) => {
				let (cmd, data) = parsed.entries().nth(0).unwrap();
				WebClient::parsePost(id, cmd.to_string(), data.clone())
			},
			Err(_) => ServerMessage::Invalid(id)
		}
	}

	fn parsePost(id: SocketAddr, cmd: String, data: json::JsonValue) -> ServerMessage
	{
		if !data.is_object()
		{
			println!("Wrong request: arguments should be provided as object with properties.");
			return ServerMessage::Invalid(id);
		}

		if cmd == "players" { return ServerMessage::PlayersList(id); }
		else if cmd == "chat"
		{
			for (section, value) in data.entries()
			{
				if section == "msg"
				{
					return ServerMessage::Chat(
						value.as_str().unwrap_or("").to_string(),
						id
					);
				}
			}
			return ServerMessage::Invalid(id);
		}
		else if cmd == "getChat"
		{
			for (section, value) in data.entries()
			{
				if section == "messagesLength"
				{
					return ServerMessage::ChatHistory(value.as_usize().unwrap_or(0), id);
				}
			}
			return ServerMessage::Invalid(id);
		}
		else if cmd == "state" { return ServerMessage::GameState(id); }
		else if cmd == "chatLength" { return ServerMessage::ChatLength(id); }
		else if cmd == "getSettings" { return ServerMessage::GetSettings(id); }
		else if cmd == "saveSettings"
		{
			let cfg = Server::getInstance().getConfig();
			for (var, value) in data.entries()
			{
				if var == "extendedPlayers"
				{
					cfg.extendedPlayers = value.as_bool().unwrap_or(false);
				}
				else if var == "port"
				{
					cfg.port = value.as_u16().unwrap_or(2018);
				}
				else if var == "tickRate"
				{
					cfg.tickRate = value.as_u8().unwrap_or(1);
					cfg.sendTime = Duration::from_secs_f32(1.0 / cfg.tickRate as f32);
					cfg.recvTime = Duration::from_secs_f32(0.5 / cfg.tickRate as f32);
				}
				else if var == "firstCP"
				{
					cfg.firstCheckpoint = value.as_str().unwrap().to_string();
				}
				else if var == "name"
				{
					cfg.name = value.as_str().unwrap().to_string();
				}
				else
				{
					cfg.setPermission(var.to_string(),
					match value.as_str().unwrap_or("")
					{
						"Разработчик" => super::Config::Permission::Developer,
						_ => super::Config::Permission::Player,
					});
				}
			}
			cfg.save();
			return ServerMessage::SaveSettings(id);
		}
		else
		{
			println!("Unknown command: {cmd}");
			return ServerMessage::Invalid(id);
		}
	}

	pub fn sendResponse(id: SocketAddr, code: WebResponse)
	{
		let c = Server::getInstance().getWebClient();
		let msg = code.build();
		for i in 0..c.tcp.len()
		{
			let tcp = &mut c.tcp[i];
			if tcp.peer_addr().unwrap() == id
			{
				match tcp.write_all(&msg)
				{
					Ok(_) => {},
					Err(x) => { println!("Error occured when sending response: {x:?}"); }
				}
				c.tcp.remove(i);
				break;
			}
		}
	}
}