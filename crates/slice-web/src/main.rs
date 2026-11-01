use std::{env, fs::File, io::Read};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args()
        .nth(1)
        .ok_or("usage: slice-playground REQUEST.json")?;
    let mut input = String::new();
    File::open(path)?
        .take(3 * 1024 * 1024 + 1)
        .read_to_string(&mut input)?;
    println!("{}", slice_web::evaluate(&input));
    Ok(())
}
