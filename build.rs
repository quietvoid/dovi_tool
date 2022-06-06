use vergen::{vergen, Config, SemverKind, ShaKind};

fn main() {
    let mut config = Config::default();

    *config.git_mut().sha_kind_mut() = ShaKind::Short;
    *config.git_mut().semver_kind_mut() = SemverKind::Lightweight;

    // Generate the instructions
    vergen(config).unwrap()
}
