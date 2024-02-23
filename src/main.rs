use std::io::Error;

use dependency_remover::{DependencyReader, NpmReader};

fn main() -> Result<(), Error> {
    let path = std::env::args().nth(1).expect("no file given");
    let mut reader = NpmReader::new(path)?;
    reader.read_unusued_deps()?;
    println!("Found dependencies: {:?}", reader.dependencies);
    println!("Found unused deps: {:?}", reader.unusued_dependencies);
    Ok(())
}
