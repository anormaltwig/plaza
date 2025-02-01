use std::io;

pub type Result<T> = std::result::Result<T, BureauError>;

#[allow(dead_code)]
#[derive(Debug)]
pub enum ErrorType {
	Io(io::Error),
	Lua(mlua::Error),
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct BureauError(ErrorType);

impl From<mlua::Error> for BureauError {
	fn from(value: mlua::Error) -> Self {
		BureauError(ErrorType::Lua(value))
	}
}

impl From<io::Error> for BureauError {
	fn from(value: io::Error) -> Self {
		BureauError(ErrorType::Io(value))
	}
}
