use std::{collections::HashMap, time::Instant};

pub struct Profiler
{
	timer: Instant,
	values: HashMap<String, f32>
}

impl Profiler
{
	pub fn new() -> Self
	{
		Self { timer: std::time::Instant::now(), values: HashMap::new() }
	}

	pub fn restart(&mut self) { self.timer = Instant::now(); }

	pub fn save(&mut self, name: String) -> f32
	{
		let x = self.timer.elapsed().as_secs_f32();
		self.values.insert(name, x);
		x
	}

	pub fn get(&self, name: String) -> f32
	{
		if let Some(x) = self.values.get(&name)
		{
			*x
		} else { 0.0 }
	}
}
