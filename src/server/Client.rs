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

	pub fn receiveTCP(&mut self) -> Option<ServerMessage>
	{
		if self.tcp.is_none() { return None; }
		let buffer = &mut [0u8; 1024];
		match self.tcp.as_mut().unwrap().read(buffer)
		{
			Ok(size) =>
			{
				if size == 0 { Some(ServerMessage::Disconnected) }
				else { Some(ServerMessage::fromRaw(&buffer[0..size])) }
			},
			Err(x) =>
			{
				match x.kind()
				{
					ErrorKind::WouldBlock => return None,
					ErrorKind::ConnectionReset => return Some(ServerMessage::Disconnected),
					_ =>
					{
						println!("{}: {x:?}", self.info.name);
						self.tcp = None;
						return Some(ServerMessage::Disconnected);
					}
				}
			}
		}
	}
}