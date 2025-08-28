#[derive(Debug, Clone)]
pub enum ServerMessage
{
	Chat(String),
	Disconnected,
	GetGameInfo(u8),
	GetPlayerInfo(u8, u8),
	SetPlayerInfo(u8, Vec<u8>),
	SetGameInfo(u8, Vec<u8>)
}

#[derive(Debug, Clone)]
pub enum ClientMessage
{
	Login(u8, String),
	Disconnected(u8),
	Chat(String),
	GameInfo(u8, Vec<u8>),
	PlayerInfo(u8, u8, Vec<u8>)
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
				].concat()
			}
			Self::Disconnected(id) =>
			{
				vec![2u8, id]
			}
			Self::Chat(text) =>
			{
				[
					&[3u8], text.as_bytes(), &[0u8]
				].concat()
			},
			Self::GameInfo(kind, raw) =>
			{
				[
					&[4u8], &[kind], raw.as_slice()
				].concat()
			},
			Self::PlayerInfo(id, kind, raw) =>
			{
				[
					&[5u8], &[id], &[kind],
					raw.as_slice()
				].concat()
			}
		}
	}
}