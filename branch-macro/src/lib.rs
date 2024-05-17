use std::fs;

use mlua::Lua;
use proc_macro::{Literal, TokenStream, TokenTree};
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
	let path = parse_macro_input!(tokens as MacroInput).0;
	let file = fs::read(&path).unwrap();

	let lua = Lua::new();
	let data = lua
		.load(file)
		.set_name(&path)
		.into_function()
		.unwrap()
		.dump(true);

	TokenTree::Literal(Literal::byte_string(&data)).into()
}
