crate::ix!();

pub fn gather_all_code_from_crates_main() -> Result<ExitCode,AppError> {

    configure_tracing();

    let effective_config = build_effective_config_from_cli()?;

    let mut output = String::new();
    for crt in effective_config.crates() {
        match process_crate_directory(crt, effective_config.criteria()) {
            Ok(res) => {
                output.push_str(&res);
            }
            Err(e) => {
                eprintln!("Error processing crate: {:?}, e: {:#?}", crt,e);
                return Ok(ExitCode::FAILURE);
            }
        }
    }

    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    if handle.write_all(output.as_bytes()).is_err() {
        return Ok(ExitCode::FAILURE);
    }

    Ok(ExitCode::SUCCESS)
}
