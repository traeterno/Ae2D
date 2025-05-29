use wrapped2d::{b2::ContactListener, dynamics::world::callbacks::ContactAccess, user_data::UserData};

use crate::ae2d::Window::Window;

use super::World::EntData;

pub struct CL;

impl CL
{
	fn HandleContact(&mut self, data: &ContactAccess<EntData>, action: bool) -> bool
	{
		let b1: Vec<&str> = data.body_a.user_data().split("-").collect();
		let f1 = data.fixture_a.user_data();
		let b2: Vec<&str> = data.body_b.user_data().split("-").collect();
		let f2 = data.fixture_b.user_data();
		if b1[0] == "interactable" && b2[0] == "entity"
		{
			self.EntityInteractable(b2[1], b1[1], f2, f1, action);
			return true;
		}
		if b2[0] == "interactable" && b1[0] == "entity"
		{
			self.EntityInteractable(b1[1], b2[1], f1, f2, action);
			return true;
		}
		if b1[0] == "entity" && b2[0] == "entity"
		{
			self.EntityEntity(b1[1], b2[1], &f1, &f2, action);
			return true;
		}

		return false;
	}

	fn EntityInteractable(&mut self, entity: &str, _object: &str, entFix: &str, _objFix: &str, action: bool)
	{
		// let ent = Window::getWorld().getEntity(entity).unwrap();
		// if entFix == "bottom" { ent.physics.touchingGround = action; }
		// if entFix == "middle" { ent.physics.touchingWall = action; }
	}

	fn EntityEntity(&mut self, _ent1: &str, _ent2: &str, _ent1fix: &str, _ent2fix: &str, _action: bool)
	{
		// 
	}
}

impl ContactListener<EntData> for CL
{
	fn begin_contact(&mut self, data: wrapped2d::dynamics::world::callbacks::ContactAccess<EntData>)
	{
		if !self.HandleContact(&data, true)
		{
			println!("Unhandled contact:\n{} {}",
				data.body_a.user_data(),
				data.body_b.user_data()
			);
		}
	}

	fn end_contact(&mut self, data: wrapped2d::dynamics::world::callbacks::ContactAccess<EntData>)
	{
		if !self.HandleContact(&data, false)
		{
			println!("Unhandled contact:\n{} {}",
				data.body_a.user_data(),
				data.body_b.user_data()
			);
		}
	}

	fn post_solve(&mut self, _: wrapped2d::dynamics::world::callbacks::ContactAccess<EntData>, _: &wrapped2d::b2::ContactImpulse) {}
	fn pre_solve(&mut self, _: wrapped2d::dynamics::world::callbacks::ContactAccess<EntData>, _: &wrapped2d::b2::Manifold) {}
}