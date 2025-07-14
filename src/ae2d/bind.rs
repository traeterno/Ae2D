use mlua::Lua;

use super::{Sprite::Sprite, Text::Text, Window::Window};

fn getSprite(id: String) -> &'static mut Sprite
{
	let mut id = id.split("_");

	match id.nth(0).unwrap()
	{
		"ui" => Window::getUI().getObject(id.nth(0).unwrap().to_string()).getSprite(),
		x => panic!("Sprite Lua: {x} not defined")
	}
}

fn getText(id: String) -> &'static mut Text
{
	let mut id = id.split("_");
	
	match id.nth(0).unwrap()
	{
		"ui" => Window::getUI().getObject(id.nth(0).unwrap().to_string()).getText(),
		x => panic!("Text Lua: {x} not defined")
	}
}

pub fn sprite(s: &Lua)
{
	let t = s.create_table().unwrap();

	let _ = t.set("draw",
	s.create_function(|s, _: ()|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		Window::getCamera().draw(spr);
		Ok(())
	}).unwrap());

	let _ = t.set("size",
	s.create_function(|s, _: ()|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		let s = spr.getFrameSize();
		Ok((s.x, s.y))
	}).unwrap());

	let _ = t.set("setTextureRect",
	s.create_function(|s, x: (f32, f32, f32, f32)|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		spr.setTextureRect(glam::vec4(x.0, x.1, x.2, x.3));
		Ok(())
	}).unwrap());

	let _ = t.set("setAnimation",
	s.create_function(|s, x: String|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		spr.setAnimation(x);
		Ok(())
	}).unwrap());

	let _ = t.set("setPosition",
	s.create_function(|s, x: (f32, f32)|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		spr.getTransformable().setPosition(glam::vec2(x.0, x.1));
		Ok(())
	}).unwrap());

	let _ = t.set("translate",
	s.create_function(|s, x: (f32, f32)|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		spr.getTransformable().translate(glam::vec2(x.0, x.1));
		Ok(())
	}).unwrap());

	let _ = t.set("getPosition",
	s.create_function(|s, _: ()|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		let x = spr.getTransformable().getPosition();
		Ok((x.x, x.y))
	}).unwrap());

	let _ = t.set("setOrigin",
	s.create_function(|s, x: (f32, f32)|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		spr.getTransformable().setOrigin(glam::vec2(x.0, x.1));
		Ok(())
	}).unwrap());

	let _ = t.set("getOrigin",
	s.create_function(|s, _: ()|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		let x = spr.getTransformable().getOrigin();
		Ok((x.x, x.y))
	}).unwrap());

	let _ = t.set("setScale",
	s.create_function(|s, x: (f32, f32)|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		spr.getTransformable().setScale(glam::vec2(x.0, x.1));
		Ok(())
	}).unwrap());

	let _ = t.set("scale",
	s.create_function(|s, x: (f32, f32)|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		spr.getTransformable().scale(glam::vec2(x.0, x.1));
		Ok(())
	}).unwrap());

	let _ = t.set("getScale",
	s.create_function(|s, _: ()|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		let x = spr.getTransformable().getScale();
		Ok((x.x, x.y))
	}).unwrap());

	let _ = t.set("setRotation",
	s.create_function(|s, x: f32|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		spr.getTransformable().setRotation(x);
		Ok(())
	}).unwrap());

	let _ = t.set("rotate",
	s.create_function(|s, x: f32|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		spr.getTransformable().rotate(x);
		Ok(())
	}).unwrap());

	let _ = t.set("getRotation",
	s.create_function(|s, _: ()|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		let x = spr.getTransformable().getRotation();
		Ok(x)
	}).unwrap());

	let _ = s.globals().set("sprite", t);
}

pub fn text(s: &Lua)
{
	let t = s.create_table().unwrap();

	let _ = t.set("draw",
	s.create_function(|s, _: ()|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		Window::getCamera().draw(txt);
		Ok(())
	}).unwrap());

	let _ = t.set("size",
	s.create_function(|s, _: ()|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		let d = txt.getDimensions();
		Ok((d.x, d.y))
	}).unwrap());

	let _ = t.set("bounds",
	s.create_function(|s, _: ()|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		let d = txt.getBounds();
		Ok((d.x, d.y, d.z, d.w))
	}).unwrap());

	let _ = t.set("setString",
	s.create_function(|s, x: String|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		txt.setString(x);
		Ok(())
	}).unwrap());

	let _ = t.set("setPosition",
	s.create_function(|s, x: (f32, f32)|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		txt.getTransformable().setPosition(glam::vec2(x.0, x.1));
		Ok(())
	}).unwrap());

	let _ = t.set("translate",
	s.create_function(|s, x: (f32, f32)|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		txt.getTransformable().translate(glam::vec2(x.0, x.1));
		Ok(())
	}).unwrap());

	let _ = t.set("getPosition",
	s.create_function(|s, _: ()|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		let x = txt.getTransformable().getPosition();
		Ok((x.x, x.y))
	}).unwrap());

	let _ = t.set("setOrigin",
	s.create_function(|s, x: (f32, f32)|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		txt.getTransformable().setOrigin(glam::vec2(x.0, x.1));
		Ok(())
	}).unwrap());

	let _ = t.set("getOrigin",
	s.create_function(|s, _: ()|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		let x = txt.getTransformable().getOrigin();
		Ok((x.x, x.y))
	}).unwrap());

	let _ = t.set("setScale",
	s.create_function(|s, x: (f32, f32)|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		txt.getTransformable().setScale(glam::vec2(x.0, x.1));
		Ok(())
	}).unwrap());

	let _ = t.set("scale",
	s.create_function(|s, x: (f32, f32)|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		txt.getTransformable().scale(glam::vec2(x.0, x.1));
		Ok(())
	}).unwrap());

	let _ = t.set("getScale",
	s.create_function(|s, _: ()|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		let x = txt.getTransformable().getScale();
		Ok((x.x, x.y))
	}).unwrap());

	let _ = t.set("setRotation",
	s.create_function(|s, x: f32|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		txt.getTransformable().setRotation(x);
		Ok(())
	}).unwrap());

	let _ = t.set("rotate",
	s.create_function(|s, x: f32|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		txt.getTransformable().rotate(x);
		Ok(())
	}).unwrap());

	let _ = t.set("getRotation",
	s.create_function(|s, _: ()|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		let x = txt.getTransformable().getRotation();
		Ok(x)
	}).unwrap());

	let _ = s.globals().set("text", t);
}