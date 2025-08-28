use std::{io::{Read, Write}, net::TcpStream};

use crate::server::Server::Server;

pub struct WebClient;

impl WebClient
{
	pub fn handle(mut tcp: TcpStream)
	{
		let _ = tcp.set_nonblocking(false);
		let _ = std::thread::Builder::new()
			.name(tcp.peer_addr().unwrap().to_string())
			.spawn(move ||
		{
			let mut buf = [0u8; 1024];
			let buf = match tcp.read(&mut buf)
			{
				Ok(size) => String::from_utf8_lossy(&buf[0..size]).to_string(),
				Err(x) => panic!("{x:#?}")
			};

			let mut req = buf.split("\r\n");
			let mut info = req.nth(0).unwrap().split(" ");
			let action = info.nth(0).unwrap();
			if action == "GET" { Self::get(tcp, info.nth(0).unwrap()); }
			else if action == "POST"
			{
				if let Ok(x) = json::parse(req.last().unwrap())
				{
					Self::post(tcp, x);
				}
			}
		});
	}

	fn get(mut tcp: TcpStream, mut path: &str)
	{
		if path == "/" { path = "/index.html"; }
		let (mimetype, bin) = match path.split(".").last().unwrap()
		{
			"html" => ("text/html", false),
			"css" => ("text/css", false),
			"js" => ("text/javascript", false),
			"png" => ("image/png", true),
			"otf" => ("application/x-font-opentype", true),
			x => panic!("Unknown file type: {x}")
		};

		let data = match bin
		{
			true => match std::fs::read(String::from("res/web") + path)
			{
				Ok(f) => f, Err(x) => panic!("{path}: {x:?}")
			},
			false => match std::fs::read_to_string(String::from("res/web") + path)
			{
				Ok(f) => f.as_bytes().to_vec(), Err(x) => panic!("{path}: {x:?}")
			}
		};

		let _ = tcp.write_all(&match data.is_empty()
		{
			true => "HTTP/1.1 404 Not Found".as_bytes().to_vec(),
			false => [(String::from("HTTP/1.1 200 OK") +
				"\r\nContent-Type: " + mimetype +
				if bin { "" } else { "; charset=UTF-8" } +
				"\r\nContent-Length: " + &data.len().to_string() +
				"\r\n\r\n").as_bytes().to_vec(), data].concat()
		});
	}

	fn post(mut tcp: TcpStream, info: json::JsonValue)
	{
		let mut msg = String::new();
		for (kind, args) in info.entries()
		{
			if kind == "chatLength"
			{
				msg = Server::getState().chatHistory.len().to_string();
			}
			if kind == "players"
			{
				let p = Server::getPlayers();
				let mut a = json::array![];
				for i in 1..=10
				{
					for (id, c) in p
					{
						if *id != i || c.info.name == "noname" { continue; }
						let _ = a.push(json::object!
						{
							id: *id,
							name: c.info.name.clone(),
							className: c.info.class.clone(),
							hp: { current: 100, max: 100 },
							mana: { current: 100, max: 100 }
						});
					}
				}
				msg = json::stringify(a);
			}
			if kind == "state"
			{
				let s = Server::getState();
				let mut a = json::array![];
				let _ = a.push(json::object!{
					title: "Сохранение",
					props: {
						"Чекпоинт": s.save.checkpoint.clone(),
						"Дата сохранения": s.save.date.clone()
					}
				});
				msg = json::stringify(a);
			}
			if kind == "getSettings"
			{
				let s = Server::getState();
				msg = json::stringify(json::object!
				{
					"Сервер": {
						extendPlayers: {
							type: "toggle",
							name: "Расширенная команда игроков",
							value: s.settings.extendPlayers
						},
						tickRate: {
							type: "range",
							name: "Частота синхронизации",
							value: s.settings.tickRate,
							props: { min: 1, max: 100 }
						},
						firstCP: {
							type: "string",
							name: "Начальный чекпоинт",
							value: s.settings.firstCP.clone()
						},
						maxItemCellSize: {
							type: "range",
							name: "Максимальное количество предметов в ячейке инвентаря",
							value: s.settings.maxItemCellSize,
							props: { min: 1, max: 255 }
						}
					}
				});
			}
			if kind == "saveSettings"
			{
				let s = Server::getState();
				for (var, value) in args.entries()
				{
					if var == "extendPlayers"
					{
						s.settings.extendPlayers = value.as_bool().unwrap();
					}
					if var == "tickRate"
					{
						s.settings.tickRate = value.as_u8().unwrap();
						s.settings.sendTime = std::time::Duration::from_secs_f32(
							1.0 / s.settings.tickRate as f32
						);
					}
					if var == "firstCP"
					{
						s.settings.firstCP = value.as_str().unwrap().to_string();
					}
					if var == "maxItemCellSize"
					{
						s.settings.maxItemCellSize = value.as_u8().unwrap();
					}
				}
				Server::reloadState();
				s.save(s.save.checkpoint.clone());
				msg = String::from("{}");
				println!("Настройки игры были изменены.");
			}
			if kind == "getChat"
			{
				let s = Server::getState();
				let mut pos = 0;
				for (var, value) in args.entries()
				{
					if var == "messagesLength"
					{
						pos = value.as_usize().unwrap();
					}
				}
				if pos > s.chatHistory.len() { pos = 0; }
				let mut a = json::array![];
				for i in pos..s.chatHistory.len()
				{
					let (user, msg) = &s.chatHistory[i];
					let _ = a.push(json::object!
					{
						user: user.clone(),
						msg: msg.clone()
					});
				}
				msg = json::stringify(a);
			}
			if kind == "chat"
			{
				let s = Server::getState();
				let mut message = String::new();
				for (var, value) in args.entries()
				{
					if var == "msg"
					{
						message = value.as_str().unwrap().to_string();
					}
				}
				s.chatHistory.push((String::from("WebClient"), message.clone()));
				msg = json::stringify(json::object!
				{
					msg: message
				});
				println!("TODO broadcast message from webclient to players");
			}
		}

		if msg.is_empty()
		{
			panic!("Unknown POST request: {info}");
		}

		let _ = tcp.write_all((
			String::from("HTTP/1.1 200 OK") +
			"\r\nContent-Type: application/json" +
			"\r\nContent-Length: " + &msg.len().to_string() +
			"\r\n\r\n" + &msg
		).as_bytes());
	}
}