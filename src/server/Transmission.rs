use crate::server::State::Account;

#[derive(Debug, Clone)]
pub enum ServerMessage
{
	Register(String),
	Chat(String),
	Disconnected,
	GetGameInfo(u8),
	GetPlayerInfo(u8),
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
	PlayerInfo(u8, Account)
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
			Self::PlayerInfo(id, info) =>
			{
				[
					&[5u8], &[id],
					&[info.color.0],
					&[info.color.1],
					&[info.color.2],
					&info.hp.to_be_bytes() as &[u8],
					info.name.as_bytes(), &[0u8],
					info.class.as_bytes(), &[0u8],
				].concat()
			}
		}
	}
}