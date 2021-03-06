//! A batteries-included binary template.
// https://willcrichton.net/notes/type-level-programming/
// TODO: remove these when ready
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
use anyhow::Result;
use error::MyError;
use validator::{Validate, ValidationError};

mod cli;
mod error;
mod flow_control;
mod sendreceive;
mod session_types;
mod utils;

fn main() -> Result<()> {
	use clap::Parser;

	let context = utils::setup()?;
	if std::env::var("DOTENV_OK").is_ok() {
		log::info!("Hello, {}!", context.args.name);
		#[cfg(feature = "some_feature")]
		log::debug!("and build info: {:#?}", context.s);
	}
	Ok(())
}
