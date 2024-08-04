use vergen_gitcl::{BuildBuilder, GitclBuilder};

fn main() {
    let mut gitcl_builder = GitclBuilder::default();
    gitcl_builder.describe(true, true, Some("[0-9]*"));

    // Generate the instructions
    if let Err(e) = gitcl_builder.build() {
        eprintln!("error occured while generating instructions: {e:?}");

        BuildBuilder::default()
            .build()
            .expect("non-git vergen should succeed");
    }
}
