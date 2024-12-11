use gather_all_code_from_crates::*;

fn main() -> Result<std::process::ExitCode,AppError> {
    Ok(gather_all_code_from_crates_main()?)
}
