#![allow(non_snake_case, static_mut_refs)]

mod server;
use server::Server::Server;

fn main()
{
	let server = Server::getInstance();
	server.setVisible(true);

	loop
	{
		server.listen();
		server.update();
	}
}