#[derive(Debug, Clone)]
pub enum ServerMessage
{
	Invalid,
	Register(String),
	Chat(String),
	Disconnected
}

impl ServerMessage
{
	pub fn fromRaw(data: &[u8]) -> Self
	{
		let mut args = Vec::from(data);
		let code = args.remove(0);

		match code
		{
			1 => Self::Register(String::from_utf8_lossy(&args).to_string()),
			2 => Self::Chat(String::from_utf8_lossy(&args).to_string()),
			3 => Self::Disconnected,
			_ => Self::Invalid
		}
	}
}

#[derive(Debug, Clone)]
pub enum ClientMessage
{
	Login(u8, String),
	Disconnected(u8),
	Chat(String)
}

impl ClientMessage
{
	pub fn toRaw(self) -> Vec<u8>
	{
		match self
		{
			Self::Login(id, name) =>
			{
				[
					&[1u8], &[id], name.as_bytes(), &[0u8]
				].concat().to_vec()
			}
			Self::Disconnected(id) =>
			{
				vec![2u8, id]
			}
			Self::Chat(text) =>
			{
				[
					&[3u8], text.as_bytes(), &[0u8]
				].concat().to_vec()
			}
		}
	}
}