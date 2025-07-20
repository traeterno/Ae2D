#![allow(non_snake_case, static_mut_refs, dead_code)]
// #![windows_subsystem = "windows"]
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