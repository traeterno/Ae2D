#![allow(non_snake_case, static_mut_refs)]
// #![windows_subsystem = "windows"]
mod server;
use server::Server::Server;

fn main()
{
	let server = Server::getInstance();
	println!("Сервер запущен. Ждём игроков...");
	server.setStarted(false);
	server.setSilent(
		std::env::args().nth(1).unwrap_or_default() == "silent"
	);
	loop { server.update(); }
}