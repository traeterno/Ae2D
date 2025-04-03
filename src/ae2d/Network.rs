use std::{io::{ErrorKind, Read, Write}, net::{SocketAddr, TcpStream, UdpSocket}};

use super::Window::Window;

pub struct Network
{
	tcp: Option<TcpStream>,
	udp: Option<UdpSocket>,
	peer: Option<SocketAddr>,
	pub playerID: u8,
	pub order: u32
}

impl Network
{
	pub fn new() -> Self
	{
		Self
		{
			tcp: None,
			udp: None,
			peer: None,
			playerID: 0,
			order: 0
		}
	}

	pub fn connectTCP(&mut self, addr: String) -> bool
	{
		let ip = addr.parse::<std::net::SocketAddr>();
		if ip.is_err() { return false; }
		let mut result = TcpStream::connect(ip.unwrap());
		if result.is_err() { println!("Failed to connect to {addr}: {}", result.unwrap_err()); return false; }
		result.as_mut().unwrap().set_nonblocking(true);
		self.tcp = Some(result.unwrap());
		self.peer = Some(self.tcp.as_ref().unwrap().peer_addr().unwrap());
		true
	}

	pub fn disconnectTCP(&mut self)
	{
		if self.tcp.is_none() { return; }
		self.send(1u8, String::new());
		self.tcp = None;
	}

	pub fn sendUDP(&mut self, req: u8, options: String)
	{
		if self.udp.is_none() { return; }
		if self.peer.is_none() { return; }
		let udp = self.udp.as_mut().unwrap();
		udp.send(&[&[req], options.as_bytes()].concat());
	}

	pub fn sendRaw(&mut self, req: u8, data: &[u8])
	{
		if self.udp.is_none() { return; }
		if self.peer.is_none() { return; }
		let udp = self.udp.as_mut().unwrap();
		udp.send(&[&[req], data].concat());
	}

	pub fn send(&mut self, req: u8, options: String)
	{
		if self.tcp.is_none() { self.sendUDP(req, options); return; }
		let tcp = self.tcp.as_mut().unwrap();
		let res = tcp.write(&[&[req as u8], options.as_bytes()].concat());
		if res.is_err()
		{
			println!("Failed to send data: {:?}", res.unwrap_err());
			self.tcp = None;
		}
	}

	pub fn receiveUDP(&mut self) -> (u8, String)
	{
		if self.udp.is_none() { return (1u8, String::new()); }
		if self.peer.is_none() { return (1u8, String::new()); }
		let udp = self.udp.as_mut().unwrap();
		let response = &mut [0u8; 1024];
		let res = udp.recv(response);
		if res.is_err() { return (255u8, String::new()); }
		let data = res.unwrap();
		return (response[0], String::from_utf8_lossy(&response[1..data]).to_string());
	}

	pub fn receiveRaw(&mut self) -> (u8, Vec<u8>)
	{
		if self.udp.is_none() { return (1u8, vec![]); }
		if self.peer.is_none() { return (1u8, vec![]); }
		let udp = self.udp.as_mut().unwrap();
		let response = &mut [0u8; 1024];
		let res = udp.recv(response);
		if res.is_err() { return (255u8, vec![]); }
		let data = res.unwrap();
		return (
			response[0],
			response[1..data].to_vec()
		);
	}

	pub fn receive(&mut self) -> (u8, String)
	{
		if self.tcp.is_none() { return self.receiveUDP(); }
		let tcp = self.tcp.as_mut().unwrap();
		let response = &mut [0u8; 1024];
		let res = tcp.read(response);
		if res.is_err()
		{
			let err = res.unwrap_err();
			if err.kind() != ErrorKind::WouldBlock
			{
				println!("Failed to receive data: {err:?}");
				self.tcp = None;
			}
			return (255u8, String::new());
		}
		let len = res.unwrap();
		if len == 0 { self.tcp = None; return (255u8, String::new()); }
		return (response[0], String::from_utf8_lossy(&response[1..len]).to_string());
	}

	pub fn bindUDP(&mut self)
	{
		let udp = UdpSocket::bind("0.0.0.0:0").unwrap();
		udp.connect(self.peer.as_ref().unwrap());

		self.playerID = Window::getVariable("PlayerID".to_string()).num as u8;

		self.udp = Some(udp);
	}
}