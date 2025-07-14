use std::net::SocketAddr;

// Incoming messages
#[derive(Debug, Clone)]
pub enum ServerMessage
{
	Invalid(SocketAddr),
	Register(String),
	Chat(String, SocketAddr),
	Disconnected,
	PlayersList(SocketAddr),
	SaveGame(String),
	ChatHistory(usize, SocketAddr),
	GameState(SocketAddr),
	ChatLength(SocketAddr),
	GetSettings(SocketAddr),
	SaveSettings(SocketAddr)
}

impl ServerMessage
{
	pub fn fromRaw(data: &[u8]) -> Self
	{
		let code = data[0];
		let mut args = Vec::from(data);
		let _ = args.remove(0);

		match code
		{
			1 => Self::Register(String::from_utf8_lossy(&args).to_string()),
			2 => Self::Chat(String::from_utf8_lossy(&args).to_string(), "0.0.0.0:0".parse().unwrap()),
			3 => Self::SaveGame(String::from_utf8_lossy(&args).to_string()),
			_ => Self::Invalid("0.0.0.0:0".parse().unwrap())
		}
	}
}

// Outcoming messages
#[derive(Debug, Clone)]
pub enum ClientMessage
{
	Login(u8, String, String),
	Disconnected(u8),
	Chat(String),
	SetPosition(u16, u16),
	GetInfo(u16, u8, String, u8)
}

impl ClientMessage
{
	pub fn toRaw(self) -> Vec<u8>
	{
		match self
		{
			Self::Login(
				id, name, class) => [
					&[1], &[id],
					name.as_bytes(), &[0],
					class.as_bytes()
				].concat().to_vec(),
			Self::Disconnected(id) => vec![2, id],
			Self::Chat(text) => [&[3], text.as_bytes()].concat().to_vec(),
			Self::SetPosition(x, y) => [&[4u8] as &[u8],
					&x.to_le_bytes(), &y.to_le_bytes()
				].concat().to_vec(),
			Self::GetInfo(udp, tickRate, checkpoint, playersCount) => [
					&[5u8] as &[u8], &udp.to_le_bytes(), &[tickRate],
					&[playersCount], checkpoint.as_bytes()
				].concat().to_vec()
		}
	}
}

#[derive(Debug, Clone)]
pub enum WebRequest { Invalid, Get(String), Post(String) }

impl WebRequest
{
	pub fn build(raw: String) -> Self
	{
		let mut data = raw.split("\n").collect::<Vec<&str>>();
		let cmd = data[0].split(" ").collect::<Vec<&str>>();
		if data.len() == 0 { return Self::Invalid; }
		while data[0] != "\r" { data.remove(0); }
		data.remove(0);
		
		if cmd[0] == "GET" { return Self::Get(cmd[1].to_string()); }
		if cmd[0] == "POST" { return Self::Post(data[0..data.len()].concat().to_string()); }
		println!("Unparsed request: {cmd:#?}");
		Self::Invalid
	}
}

// https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Status
#[derive(Debug, Clone)]
pub enum WebResponse
{
	Ok(String, String),
	OkRaw(Vec<u8>, String),
	MovedPermanently(String),
	NotFound
}

impl WebResponse
{
	pub fn build(self) -> Vec<u8>
	{
		match self
		{
			Self::Ok(data, filetype) =>
				(String::from("HTTP/1.1 200 OK") +
				"\r\nContent-Type: " + &filetype + "; charset=UTF-8" +
				"\r\nContent-Length: " + &data.len().to_string() +
				"\r\n\r\n" + &data).as_bytes().to_vec(),
			Self::OkRaw(data, filetype) =>
				[(String::from("HTTP/1.1 200 OK") +
				"\r\nContent-Type: " + &filetype +
				"\r\nContent-Length: " + &data.len().to_string() +
				"\r\n\r\n").as_bytes(), &data].concat().to_vec(),
			Self::MovedPermanently(path) =>
				(String::from("HTTP/1.1 301 Moved Permanently") +
				"\r\nLocation: " + &path).as_bytes().to_vec(),
			Self::NotFound => String::from("HTTP/1.1 404 Not Found").as_bytes().to_vec(),
		}
	}
}