use std::{io::{ErrorKind, Read, Write}, net::{SocketAddr, TcpStream}};

use super::Transmission::{ClientMessage, ServerMessage};

pub struct Client
{
	pub id: u8,
	pub tcp: Option<TcpStream>,
	pub name: String,
	pub class: String,
	pub udp: Option<SocketAddr>
}

impl Client
{
	pub fn default() -> Self
	{
		Self
		{
			id: 0,
			tcp: None,
			name: String::new(),
			class: String::new(),
			udp: None
		}
	}
	pub fn connect(tcp: TcpStream, id: u8, name: String, class: String) -> Self
	{
		let _ = tcp.set_nodelay(true);
		let _ = tcp.set_nonblocking(true);
		
		Self
		{
			id,
			tcp: Some(tcp),
			name: name.clone(),
			class: class.clone(),
			udp: None
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
						println!("Error occured on player {}: {x:?}", self.name);
						self.tcp = None;
						return Some(ServerMessage::Disconnected);
					}
				}
			}
		}
	}
}