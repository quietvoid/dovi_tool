use vergen::EmitBuilder;

fn main() {
    let mut emit_builder = EmitBuilder::builder();
    emit_builder.fail_on_error();
    emit_builder.git_describe(true, true, Some("[0-9]*"));

    // Generate the instructions
    if let Err(e) = emit_builder.emit() {
        eprintln!("error occured while generating instructions: {e:?}");
        EmitBuilder::builder()
            .emit()
            .expect("non-git vergen should succeed");
    }
}
