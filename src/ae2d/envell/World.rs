use std::{thread::JoinHandle, time::{Duration, Instant}};

use wrapped2d::{b2, user_data::UserDataTypes};

use crate::ae2d::{Assets, Window::Window};

use super::{ContactListener::CL, DebugDraw::DebugDraw, Entity::Entity};

pub const m2p: f32 = 32.0;

pub struct EntData;

impl UserDataTypes for EntData
{
	type BodyData = String;
	type FixtureData = String;
	type JointData = ();
}

pub struct World
{
	ents: Vec<Entity>,
	pub currentEnt: *mut Entity,
	pub world: b2::World<EntData>,
	debugDraw: DebugDraw,
	duration: Duration,
	lastSent: Instant,
	netHandle: Option<JoinHandle<()>>,
}

impl World
{
	pub fn new() -> Self
	{
		Self
		{
			ents: vec![],
			currentEnt: std::ptr::null::<Entity>() as *mut _,
			world: b2::World::new(&b2::Vec2 { x: 0.0, y: 0.0 }),
			debugDraw: DebugDraw {},
			duration: Duration::from_millis(10),
			lastSent: Instant::now(),
			netHandle: None,
		}
	}

	pub fn load(&mut self, path: String)
	{
		let res = Assets::readXML(path);
		if res.is_none() { return; }
		let doc = res.unwrap();

		if let Some(size) = doc.att_opt("camSize")
		{
			let dim: Vec<&str> = size.split(" ").collect();
			if dim.len() == 2
			{
				Window::getCamera().setSize(glam::vec2(dim[0].parse().unwrap(), dim[1].parse().unwrap()));
			}
		}
		if let Some(gravity) = doc.att_opt("gravity")
		{
			let v: Vec<&str> = gravity.split(" ").collect();
			if v.len() == 2
			{
				self.world.set_gravity(&b2::Vec2 { x: v[0].parse().unwrap(), y: v[1].parse().unwrap() });
			}
		}

		for element in doc.elements()
		{
			let name = element.name().local_part();
			if name == "entity"
			{
				self.ents.push(Entity::parse(element));
			}
		}

		self.world.set_contact_listener(Box::new(CL {}));
	}

	pub fn destroyEntity(&mut self, id: usize)
	{
		self.world.destroy_body(self.ents[id].physics.body);
		self.ents.swap_remove(id);
	}
	
	pub fn update(&mut self)
	{
		self.world.step(Window::getDeltaTime(), 12, 8);

		self.currentEnt = std::ptr::null::<Entity>() as *mut _;
		for id in 0..self.ents.len()
		{
			if self.ents[id].destroyed { self.destroyEntity(id); continue; }
			let ent = &mut self.ents[id];
			self.currentEnt = ent;
			ent.handleRequest();
			ent.update();
		}

		let net = Window::getNetwork();
		if net.playerID == 0 { return; }
		if self.lastSent.elapsed() > self.duration
		{
			let ent = self.getEntity(&Window::getVariable("playerEnt".to_string()).string);
			if ent.is_none() { return; }
			let ent = ent.unwrap();
			net.sendRaw(
				6,
				&ent.netPack()
			);
			self.lastSent = Instant::now();
		}
		if self.netHandle.is_none() { self.netHandle = Some(std::thread::spawn(World::updateNetwork)); }
	}

	pub fn updateNetwork()
	{
		let world = Window::getWorld();
		let net = Window::getNetwork();

		loop
		{
			let (req, b) = net.receiveRaw();
			if b.len() == 0 { continue; }
			let order = u32::from_le_bytes([b[0], b[1], b[2], b[3]]);
			if order < net.order { continue; }
			net.order = order;
			if req == 6
			{
				for i in 0..((b.len() - 4) / 17)
				{
					let offset = 4 + i * 17;
					let id = b[offset];
					if net.playerID == id { continue; }
					let ch = Window::getVariable("p".to_string() + &id.to_string()).string;
					let ent = world.getEntity(&ch);
					if ent.is_none() { continue; }
					ent.unwrap().request([
						f32::from_le_bytes([b[offset + 1], b[offset + 2], b[offset + 3], b[offset + 4]]),
						f32::from_le_bytes([b[offset + 5], b[offset + 6], b[offset + 7], b[offset + 8]]),
						f32::from_le_bytes([b[offset + 9], b[offset + 10], b[offset + 11], b[offset + 12]]),
						f32::from_le_bytes([b[offset + 13], b[offset + 14], b[offset + 15], b[offset + 16]])
					]);
				}
			}
		}
	}
	
	pub fn render(&mut self)
	{
		let cam = Window::getCamera();
		cam.toggleCameraTransform(true);
		cam.update();

		for ent in &mut self.ents
		{
			cam.draw(ent);
		}

		self.world.draw_debug_data(&mut self.debugDraw, b2::DrawFlags::DRAW_SHAPE);
	}

	pub fn getEntity(&mut self, name: &str) -> Option<&mut Entity>
	{
		for ent in &mut self.ents
		{
			if *ent.getName() == name { return Some(ent); }
		}
		None
	}
}