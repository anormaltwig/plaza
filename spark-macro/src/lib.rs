#![feature(proc_macro_span)]
#![feature(track_path)]

use std::fs;

use mlua::Lua;
use proc_macro::{tracked_path, Literal, Span, TokenStream, TokenTree};
use syn::{
	parse::{Parse, ParseStream},
	parse_macro_input, LitStr,
};

struct MacroInput(String);

impl Parse for MacroInput {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let s: LitStr = input.parse()?;
		Ok(Self(s.value()))
	}
}

/// Load lua file and convert it to bytecode.
#[proc_macro]
pub fn include_lua(tokens: TokenStream) -> TokenStream {
	let target = parse_macro_input!(tokens as MacroInput).0;

	let mut path = Span::call_site()
		.source_file()
		.path()
		.parent()
		.unwrap()
		.to_path_buf();
	path.push(target);

	let file = fs::read(&path).expect("Failed to read file.");
	tracked_path::path(path.to_str().unwrap());

	let lua = Lua::new();
	let data = lua
		.load(file)
		.set_name(path.to_str().unwrap())
		.into_function()
		.unwrap()
		.dump(true);

	TokenTree::Literal(Literal::byte_string(&data)).into()
}
