use std::{collections::HashMap, io::{Read, Write}, net::{TcpListener, TcpStream}};

use base64::{prelude::BASE64_STANDARD, Engine};

use crate::server::Server::Server;

pub struct WebClient
{
	ws: Option<TcpStream>
}

impl WebClient
{
	pub fn init() -> Self
	{
		let _ = std::thread::Builder::new().name("WebListener".to_string()).spawn(||
		{
			std::thread::sleep(std::time::Duration::from_secs(1));
			let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
			for c in listener.incoming()
			{
				if let Ok(s) = c { Self::handle(s); }
			}
		});
		Self { ws: None }
	}

	fn handle(mut tcp: TcpStream)
	{
		let _ = tcp.set_nonblocking(false);
		let _ = tcp.set_nodelay(true);
		let _ = std::thread::Builder::new()
			.name(tcp.peer_addr().unwrap().to_string())
			.spawn(move ||
		{
			let mut buf = [0u8; 1024];
			let buf: Vec<String> = match tcp.read(&mut buf)
			{
				Ok(size) => String::from_utf8_lossy(&buf[0..size]).to_string(),
				Err(x) => panic!("{x:#?}")
			}.split("\r\n").map(|x| x.to_string()).collect();

			let mut args = HashMap::new();
			for x in &buf
			{
				if x.find(": ").is_some()
				{
					let a: Vec<&str> = x.split(": ").collect();
					args.insert(a[0].to_string(), a[1].to_string());
				}
			}

			let mut info = buf[0].split(" ");
			let action = info.nth(0).unwrap();
			if action == "GET" { Self::get(tcp, info.nth(0).unwrap(), args); }
			else if action == "POST"
			{
				if let Ok(x) = json::parse(buf.last().unwrap())
				{
					Self::post(tcp, x);
				}
			}
		});
	}

	fn get(mut tcp: TcpStream, mut path: &str, args: HashMap<String, String>)
	{
		if path == "/ws" { Self::websocket(tcp, args); return; }
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
			if kind == "saveSettings"
			{
				let s = Server::getState();
				for (var, value) in args.entries()
				{
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
				s.save(s.save.checkpoint.clone());
				msg = String::from("{}");
				println!("Настройки игры были изменены.");
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

	fn websocket(mut tcp: TcpStream, args: HashMap<String, String>)
	{
		let wc = Server::getWC();
		if wc.ws.is_some()
		{
			let _ = tcp.write_all("HTTP/1.1 403 Forbidden\r\n\r\n".as_bytes());
			return;
		}
		let addr = tcp.peer_addr().unwrap();
		println!("Переключаем {addr} на режим WebSocket.");
		let key = args.get(&String::from("Sec-WebSocket-Key"))
			.expect("Ключ безопасности отсутствует.").to_owned();
		println!("Ключ от {addr}: {key}");
		let magic = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
		let enc = BASE64_STANDARD.encode(
			sha1_smol::Sha1::from(key + magic).digest().bytes()
		);
		println!("Ответный ключ: {enc}");
		if let Err(x) = tcp.write_all((
			String::from("HTTP/1.1 101 Switching Protocols") +
			"\r\nUpgrade: websocket\r\nConnection: Upgrade" +
			"\r\nSec-WebSocket-Accept: " + &enc + "\r\n\r\n"
		).as_bytes()) { panic!("{x:?}"); }

		let s = Server::getState();

		wc.ws = Some(tcp);

		wc.send("state", s.jsonState());
		wc.send("getSettings", s.jsonSettings());

		let tcp = Server::getWC().ws.as_mut().unwrap();
		
		'running: loop
		{
			let mut v = [0u8; 1024];
			let size = match tcp.read(&mut v)
			{
				Ok(size) => size,
				Err(x) => panic!("{x:?}")
			};
			if size == 0 { break 'running; }
			println!("{addr}: получено {size} байт");
			let v = &v[0..size];
			let isFinal = v[0] & 0b10_00_00_00 == 128;
			let kind = v[0] & 0b00_00_11_11;
			if kind == 8 { break 'running; }
			let (payloadLength, offset) = {
				let check =  v[1] & 0b01_11_11_11;
				if check <= 125 { (check as u64, 2) }
				else if check == 126 { (u16::from_be_bytes([v[2], v[3]]) as u64, 4) }
				else { (u64::from_be_bytes([
					v[2], v[3], v[4], v[5],
					v[6], v[7], v[8], v[9]
				]), 10) }
			};
			let key = [
				v[offset], v[offset + 1], v[offset + 2], v[offset + 3]
			];
			println!("{addr}: длина - {payloadLength}, ключ - {key:?}");
			if !isFinal { panic!("Невозможно обработать более 1 пакета."); }
			let mut raw = vec![];
			for i in 0..payloadLength as usize
			{
				raw.push(v[offset + 4 + i] ^ key[i % 4]);
			}
			let msg = json::parse(
				&String::from_utf8_lossy(&raw).to_string()
			).unwrap();
			let (msg, data) = msg.entries().nth(0).unwrap();
			println!("{addr}: {msg} - {data}");
			match msg
			{
				"chatMessages" =>
				{
					wc.send(
						"chatMessages",
						s.jsonChatHistory(
							data.entries().nth(0).unwrap().1.as_usize().unwrap()
						)
					);
				}
				"newMessage" =>
				{
					Server::getInstance().newMessage(
						String::from("WebClient"),
						data.entries().nth(0).unwrap().1.as_str().unwrap().to_string()
					);
				}
				"saveSettings" =>
				{
					for (var, value) in data.entries()
					{
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
					s.save(s.save.checkpoint.clone());
					println!("Настройки игры были изменены.");
				}
				x => panic!("Неизвестный пакет: {x}")
			}
		}

		wc.ws = None;

		// 	if playerUpdater.elapsed().as_secs_f32() > 0.5
		// 	{
		// 		let p = Server::getPlayers();
		// 		let mut arr = json::array![];
		// 		for i in 1..=5
		// 		{
		// 			for (id, c) in p
		// 			{
		// 				if *id != i || c.info.name == "noname" { continue; }
		// 				let _ = arr.push(json::object!
		// 				{
		// 					id: *id,
		// 					name: c.info.name.clone(),
		// 					className: c.info.class.clone(),
		// 					hp: { current: c.info.hp, max: 100 },
		// 					mana: { current: 100, max: 100 }
		// 				});
		// 			}
		// 		}

		// 		dataType = String::from("players");
		// 		data = arr;
		// 		playerUpdater = Instant::now();
		// 	}
	}

	pub fn send(&mut self, kind: &str, data: json::JsonValue)
	{
		if self.ws.is_none() { return; }
		let msg = json::stringify(json::object!{
			type: kind,
			data: data
		});

		let len =
			if msg.len() <= 125 { vec![msg.len() as u8] }
			else { [&[126u8] as &[u8], &(msg.len() as u16).to_be_bytes()].concat() };
		
		if let Err(x) = self.ws.as_mut().unwrap().write_all(&[
			&[0b10_00_00_01 as u8],
			len.as_slice(),
			msg.as_bytes()
		].concat()) { panic!("{x:?}"); }
	}
}