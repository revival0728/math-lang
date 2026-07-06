use anyhow::Error;
use vergen::{Build, Cargo, Emitter, Rustc};

fn main() -> Result<(), Error> {
    let build = Build::builder().build_date(true).build();
    let cargo = Cargo::builder().target_triple(true).build();
    let rustc = Rustc::builder().semver(true).build();

    Emitter::default()
        .add_instructions(&build)?
        .add_instructions(&cargo)?
        .add_instructions(&rustc)?
        .emit()?;

    Ok(())
}
