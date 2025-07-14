#![allow(non_snake_case, static_mut_refs)]
mod ae2d;
mod server;

use ae2d::Window::Window;

fn main()
{
	Window::init();

	Window::resetDT();
	while Window::isOpen()
	{
		Window::update();
	}
}