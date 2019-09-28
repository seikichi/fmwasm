use std::error::Error;
use std::io::{self, Read, Write};

use fmwasm::*;

fn main() -> Result<(), Box<dyn Error>> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;

    let index = FMIndex::new(&buffer);
    let encoded = bincode::serialize(&index)?;
    io::stdout().write_all(&encoded[..])?;

    Ok(())
}
