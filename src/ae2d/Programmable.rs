#[derive(Clone)]
pub struct Variable
{
	pub num: f32,
	pub string: String
}

impl Default for Variable
{
	fn default() -> Self
	{
		Self
		{
			num: 0.0,
			string: String::new()
		}
	}
}

impl Variable
{
	pub fn num(x: f32) -> Self
	{
		Self
		{
			num: x,
			string: String::new()
		}
	}

	pub fn str(x: String) -> Self
	{
		Self
		{
			num: 0.0,
			string: x
		}
	}
}

pub type Programmable = std::collections::HashMap<String, Variable>;