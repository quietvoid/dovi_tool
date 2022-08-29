use vergen::{vergen, Config, SemverKind, ShaKind};

fn main() {
    let mut config = Config::default();

    *config.git_mut().sha_kind_mut() = ShaKind::Short;
    *config.git_mut().semver_kind_mut() = SemverKind::Lightweight;

    // Generate the instructions
    if let Err(e) = vergen(config) {
        eprintln!("error occured while generating instructions: {:?}", e);

        config = Config::default();
        *config.git_mut().enabled_mut() = false;

        vergen(config).expect("non-git vergen should succeed");
    }
}
