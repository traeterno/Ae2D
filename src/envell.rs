#![allow(non_snake_case, static_mut_refs)]
#![windows_subsystem = "windows"]
mod server;
use server::Server::Server;

fn main()
{
	let server = Server::getInstance();
	server.setStarted(false);
	server.setSilent(std::env::args().nth(1).unwrap() == "silent");
	loop { server.update(); }
}