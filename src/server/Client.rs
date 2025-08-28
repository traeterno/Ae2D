use std::{io::{ErrorKind, Read, Write}, net::{SocketAddr, TcpStream}};

use crate::server::State::Account;

use super::Transmission::{ClientMessage, ServerMessage};

pub struct Client
{
	pub tcp: Option<TcpStream>,
	pub udp: Option<SocketAddr>,
	pub info: Account,
	pub state: [u8; 9]
}

impl Client
{
	pub fn default() -> Self
	{
		Self
		{
			tcp: None,
			udp: None,
			info: Account::default(),
			state: [0u8; 9]
		}
	}

	pub fn connect(tcp: TcpStream, info: Account) -> Self
	{
		let _ = tcp.set_nodelay(true);
		let _ = tcp.set_nonblocking(true);
		
		Self
		{
			tcp: Some(tcp),
			udp: None,
			info,
			state: [0u8; 9]
		}
	}

	pub fn sendTCP(&mut self, msg: ClientMessage)
	{
		if self.tcp.is_none() { return; }
		let _ = self.tcp.as_mut().unwrap().write_all(&msg.toRaw());
	}

	pub fn receiveTCP(&mut self) -> Vec<ServerMessage>
	{
		if self.tcp.is_none() { return vec![]; }
		let buffer = &mut [0u8; 1024];
		match self.tcp.as_mut().unwrap().read(buffer)
		{
			Ok(size) =>
			{
				if size == 0 { vec![ServerMessage::Disconnected] }
				else { Self::parse(&buffer[0..size]) }
			},
			Err(x) =>
			{
				match x.kind()
				{
					ErrorKind::WouldBlock => return vec![],
					ErrorKind::ConnectionReset => return vec![ServerMessage::Disconnected],
					_ =>
					{
						println!("{}: {x:?}", self.info.name);
						self.tcp = None;
						return vec![ServerMessage::Disconnected];
					}
				}
			}
		}
	}

	fn parse(data: &[u8]) -> Vec<ServerMessage>
	{
		let mut out = vec![];
		let mut current = 0;
		while current < data.len()
		{
			match data[current]
			{
				1 =>
				{
					let msg = {
						let mut len = 0;
						while data[current + 1 + len] != 0 { len += 1; }
						String::from_utf8_lossy(
							&data[current + 1..current + 1 + len]
						).to_string()
					};
					current += 1 + msg.len() + 1;
					out.push(ServerMessage::Chat(msg));
				}
				2 =>
				{
					current += 1;
					out.push(ServerMessage::Disconnected);
				}
				3 =>
				{
					out.push(ServerMessage::GetGameInfo(data[current + 1]));
					current += 2;
				}
				4 =>
				{
					out.push(ServerMessage::GetPlayerInfo(data[current + 1], data[current + 2]));
					current += 3;
				}
				5 =>
				{
					let raw = {
						let mut len = 0;
						while data[current + 2 + len] != 0 { len += 1; }
						&data[current + 2..current + 2 + len]
					};
					if data[current + 1] == 2
					{
						out.push(ServerMessage::SetPlayerInfo(
							2, vec![
								data[current + 2],
								data[current + 3],
								data[current + 4]
							]
						));
						current += 6;
					}
					else if data[current + 1] == 3
					{
						out.push(ServerMessage::SetPlayerInfo(
							3, vec![
								data[current + 2],
								data[current + 3]
							]
						));
						current += 5;
					}
					else
					{
						out.push(ServerMessage::SetPlayerInfo(
							data[current + 1],
							raw.to_vec()
						));
						current += 3 + raw.len();
					}
				}
				6 =>
				{
					let raw = {
						let mut len = 0;
						while data[current + 2 + len] != 0 { len += 1; }
						&data[current + 2..current + 2 + len]
					};
					out.push(ServerMessage::SetGameInfo(
						data[current + 1],
						raw.to_vec()
					));
					current += 3 + raw.len();
				}
				x =>
				{
					println!("Invalid byte: {x}");
					current += 1;
				}
			}
		}
		out
	}
}