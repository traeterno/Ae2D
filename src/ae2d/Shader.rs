use std::ffi::CString;

pub struct Shader
{
	vertex: u32,
	fragment: u32,
	program: u32
}

impl Shader
{
	pub fn new() -> Self
	{
		Self
		{
			vertex: 0,
			fragment: 0,
			program: 0
		}
	}

	fn compile(p: &str, t: gl::types::GLenum) -> u32
	{
		let res = crate::ae2d::Assets::readFile(p.to_string());
		if res.is_none()
		{
			println!("Failed to load shader from {p}");
			return 0;
		}

		let code = std::ffi::CString::new(res.unwrap()).unwrap();

		unsafe
		{
			let shader = gl::CreateShader(t);
			gl::ShaderSource(
				shader,
				1,
				&code.as_ptr(),
				std::ptr::null()
			);
			gl::CompileShader(shader);
			let mut status = 0;
			gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
			if status == 0
			{
				let mut len = 0;
				gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
				let mut buf: Vec<u8> = Vec::with_capacity(len as usize + 1);
				buf.extend([b' '].iter().cycle().take(len as usize));
				let error = std::ffi::CString::from_vec_unchecked(buf);
				gl::GetShaderInfoLog(shader, len, std::ptr::null_mut(), error.as_ptr() as *mut i8);
				println!("Failed to compile shader from {p}:\n{}", error.to_str().unwrap());
			}
			shader
		}
	}

	fn link(&mut self)
	{
		if self.program != 0 { unsafe { gl::DeleteProgram(self.program); } }
		unsafe
		{
			self.program = gl::CreateProgram();
			if self.vertex != 0 { gl::AttachShader(self.program, self.vertex); }
			if self.fragment != 0 { gl::AttachShader(self.program, self.fragment); }
			gl::LinkProgram(self.program);
			let mut status = 0;
			gl::GetProgramiv(self.program, gl::LINK_STATUS, &mut status);
			if status == 0
			{
				let mut infoLog = [0; 512];
				let mut written = 0;
				gl::GetProgramInfoLog(self.program, 512, &mut written, infoLog.as_mut_ptr());
				println!("Failed to link shader:\n{}", String::from_raw_parts(infoLog.as_mut_ptr() as *mut u8, written as usize, 512));
			}
			if self.vertex != 0
			{
				gl::DetachShader(self.program, self.vertex);
				gl::DeleteShader(self.vertex);
			}
			if self.fragment != 0
			{
				gl::DetachShader(self.program, self.fragment);
				gl::DeleteShader(self.fragment);
			}
		}
	}

	pub fn load(&mut self, vertex: &str, fragment: &str)
	{
		self.vertex = Shader::compile(vertex, gl::VERTEX_SHADER);
		self.fragment = Shader::compile(fragment, gl::FRAGMENT_SHADER);
		self.link();
	}

	pub fn activate(&mut self)
	{
		unsafe
		{
			gl::UseProgram(self.program);
		}
	}

	pub fn setInt(&mut self, name: &str, value: i32)
	{
		let cn = CString::new(name).unwrap();
		unsafe
		{
			gl::Uniform1i(gl::GetUniformLocation(self.program, cn.as_ptr()), value);
		}
	}

	pub fn setFloat(&mut self, name: &str, value: f32)
	{
		let cn = CString::new(name).unwrap();
		unsafe
		{
			gl::Uniform1f(gl::GetUniformLocation(self.program, cn.as_ptr()), value);
		}
	}

	pub fn setMat4(&mut self, name: &str, value: &[f32; 16])
	{
		let cn = CString::new(name).unwrap();
		unsafe
		{
			gl::UniformMatrix4fv(
				gl::GetUniformLocation(self.program, cn.as_ptr()),
				1,
				gl::FALSE,
				value.as_ptr()
			);
		}
	}

	pub fn setVec3(&mut self, name: &str, value: [f32; 3])
	{
		let cn = CString::new(name).unwrap();
		unsafe
		{
			gl::Uniform3f(
				gl::GetUniformLocation(self.program, cn.as_ptr()),
				*value.get(0).unwrap(),
				*value.get(1).unwrap(),
				*value.get(2).unwrap()
			);
		}
	}

	pub fn setVec4(&mut self, name: &str, value: [f32; 4])
	{
		let cn = CString::new(name).unwrap();
		unsafe
		{
			gl::Uniform4f(
				gl::GetUniformLocation(self.program, cn.as_ptr()),
				*value.get(0).unwrap(),
				*value.get(1).unwrap(),
				*value.get(2).unwrap(),
				*value.get(3).unwrap()
			);
		}
	}

	pub fn setBool(&mut self, name: &str, value: bool)
	{
		let cn = CString::new(name).unwrap();
		unsafe
		{
			gl::Uniform1i(
				gl::GetUniformLocation(self.program, cn.as_ptr()),
				value as i32
			);
		}
	}

	pub fn getInt(&self, name: &str) -> i32
	{
		let cn = CString::new(name).unwrap();
		let mut out = 0;
		unsafe
		{
			gl::GetUniformiv(
				self.program,
				gl::GetUniformLocation(self.program, cn.as_ptr()),
				&mut out
			);
		}
		out
	}

	pub fn isLoaded(&mut self) -> bool { self.program != 0 }
}

impl Drop for Shader
{
	fn drop(&mut self)
	{
		if self.program != 0 { unsafe { gl::DeleteProgram(self.program); } }
	}
}