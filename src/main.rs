use std::error::Error;

type Result<T> = anyhow::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    unimplemented!()
}
