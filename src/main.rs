use std::{env, fs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Replace with clap
    let filename = env::args().nth(1).ok_or_else(|| "No filename provided")?;
    let src = fs::read_to_string(filename)?;
    let diagram = pest_railroad::generate_diagram(&src)?;

    println!("{diagram}");
    Ok(())
}
