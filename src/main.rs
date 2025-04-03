#![allow(dead_code, non_snake_case, non_upper_case_globals, unused_must_use, static_mut_refs)]
mod ae2d;

use ae2d::Window::Window;

fn main()
{
	Window::init();

	println!("{}", Window::getGL());

	Window::resetDT();
	while Window::isOpen()
	{
		Window::update();
		Window::display();
	}
}
