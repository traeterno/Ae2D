use std::{io::{Read, Write}, net::TcpStream};

use mlua::{Error, Lua, Table};

use super::Window::Window;

pub struct Network
{
	tcp: Option<TcpStream>,
	name: String,
	class: String,
	id: u8,
	pub order: u32,
}

impl Network
{
	pub fn new() -> Self
	{
		Self
		{
			tcp: None,
			name: String::new(),
			class: String::new(),
			id: 0,
			order: 0
		}
	}

	pub fn initLua(script: &Lua)
	{
		let table = script.create_table().unwrap();

		table.set("connectTCP", script.create_function(Network::connectTCP).unwrap());
		table.set("receiveTCP", script.create_function(Network::receiveTCP).unwrap());
		table.set("getName", script.create_function(Network::getName).unwrap());
		table.set("setName", script.create_function(Network::setName).unwrap());
		table.set("getID", script.create_function(Network::getID).unwrap());
		table.set("setID", script.create_function(Network::setID).unwrap());
		table.set("getClass", script.create_function(Network::getClass).unwrap());
		table.set("setClass", script.create_function(Network::setClass).unwrap());
		table.set("sendTextTCP", script.create_function(Network::sendTextTCP).unwrap());
		
		script.globals().set("network", table);
	}

	pub fn connectTCP(_: &Lua, addr: String) -> Result<bool, Error>
	{
		let net = Window::getNetwork();
		let tcp = TcpStream::connect(addr);
		if tcp.is_err()
		{
			println!("Failed to connect TCP: {:?}", tcp.unwrap_err());
			return Ok(false);
		}
		let tcp = tcp.unwrap();
		tcp.set_nonblocking(true);
		net.tcp = Some(tcp);
		Ok(true)
	}

	pub fn receiveTCP(script: &Lua, _: ()) -> Result<(i32, Table), Error>
	{
		let table = script.create_table().unwrap();
		let tcp = &mut Window::getNetwork().tcp;
		if tcp.is_none() { return Ok((0, table)); }
		let tcp = tcp.as_mut().unwrap();

		let buffer = &mut [0u8; 1024];
		match tcp.read(buffer)
		{
			Ok(size) =>
			{
				for i in 1..size
				{
					table.raw_push(buffer[i]);
				}
				Ok((buffer[0] as i32, table))
			},
			Err(_) => Ok((0, table))
		}
	}

	pub fn sendTextTCP(_: &Lua, data: (u8, String)) -> Result<(), Error>
	{
		let tcp = &mut Window::getNetwork().tcp;
		if tcp.is_none() { return Ok(()); }
		let tcp = tcp.as_mut().unwrap();
		let _ = tcp.write(&[&[data.0], data.1.as_bytes()].concat());
		Ok(())
	}

	pub fn setName(_: &Lua, name: String) -> Result<(), Error>
	{
		Window::getNetwork().name = name;
		Ok(())
	}

	pub fn getName(_: &Lua, _: ()) -> Result<String, Error>
	{
		Ok(Window::getNetwork().name.clone())
	}

	pub fn setID(_: &Lua, id: i32) -> Result<(), Error>
	{
		Window::getNetwork().id = id as u8;
		Ok(())
	}

	pub fn getID(_: &Lua, _: ()) -> Result<i32, Error>
	{
		Ok(Window::getNetwork().id as i32)
	}

	pub fn setClass(_: &Lua, class: String) -> Result<(), Error>
	{
		Window::getNetwork().class = class;
		Ok(())
	}

	pub fn getClass(_: &Lua, _: ()) -> Result<String, Error>
	{
		Ok(Window::getNetwork().class.clone())
	}
}