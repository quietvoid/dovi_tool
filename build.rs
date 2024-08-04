use anyhow::Result;
use vergen_gitcl::{BuildBuilder, Emitter, GitclBuilder};

fn main() -> Result<()> {
    let mut emitter = Emitter::default();

    let gitcl_res = GitclBuilder::default()
        .describe(true, true, Some("[0-9]*"))
        .build();

    if let Ok(gitcl) = gitcl_res {
        emitter.add_instructions(&gitcl)?;
    } else {
        let build = BuildBuilder::default()
            .build()
            .expect("non-git vergen should succeed");
        emitter.add_instructions(&build)?;
    }

    // Generate the instructions
    emitter.emit()?;

    Ok(())
}
