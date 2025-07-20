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

pub type Programmable = std::collections::HashMap<String, Variable>;