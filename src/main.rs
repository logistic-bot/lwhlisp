use color_eyre::{eyre::eyre, Result};

mod atom;

fn main() -> Result<()> {
    color_eyre::install()?;
    println!("Hello, world!");
    Ok(())
}
