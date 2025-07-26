use std::{collections::HashMap, time::Instant};

pub struct Voting
{
	topic: String,
	options: Vec<String>,
	votes: HashMap<u8, u8>,
	timeout: f32,
	timer: Instant
}

// TODO block new votings until the last one finishes

impl Voting
{
	pub fn new() -> Self
	{
		Self
		{
			topic: String::new(),
			options: vec![],
			votes: HashMap::new(),
			timeout: 0.0,
			timer: Instant::now()
		}
	}

	pub fn start(&mut self, topic: String, opt: Vec<String>, time: f32)
	{
		self.topic = topic;
		self.options = opt;
		self.timeout = time;
		self.votes.clear();
		self.timer = Instant::now();
	}

	pub fn vote(&mut self, selection: u8)
	{
		let x = *self.votes.get(&selection).unwrap_or(&0);
		self.votes.insert(selection, x + 1);
	}

	pub fn active(&self) -> bool { !self.topic.is_empty() }

	pub fn finished(&self) -> bool
	{
		self.timer.elapsed().as_secs_f32() >= self.timeout
	}

	pub fn getResult(&self) -> (String, u8)
	{
		println!("{:#?}", self.votes);
		let mut cur = (0, 0);
		for x in &self.votes
		{
			if *x.1 > cur.1 { cur = (*x.0, *x.1); }
		}
		(self.options[cur.0 as usize].clone(), cur.1)
	}

	pub fn getTopic(&self) -> String { self.topic.clone() }
}