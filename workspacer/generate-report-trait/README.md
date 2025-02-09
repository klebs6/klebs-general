# Generate Report

`generate-report` is a minimal crate that defines a trait for generating reports. The trait, `GenerateReport`, allows you to standardize report creation by specifying associated types for the report output and any errors that might occur during report generation.

## Features

- **Lightweight Interface:** A single trait that you can implement for your own types.
- **Flexible:** Use associated types to tailor the report and error types to your application.
- **Easy Integration:** Designed to fit into larger systems that require a reporting interface.

## Usage

Add `generate-report` to your `Cargo.toml` dependencies:

```toml
[dependencies]
generate-report = "0.1.0"
```

Implement the trait for your custom type:

```rust
use generate_report::GenerateReport;

struct MyReporter;

#[derive(Debug)]
enum MyError {
    ReportError(String),
}

impl GenerateReport for MyReporter {
    type Report = String;
    type Error = MyError;

    fn generate_report(&self) -> Result<Self::Report, Self::Error> {
        // Generate and return your report here.
        Ok("This is my generated report".to_string())
    }
}

fn main() {
    let reporter = MyReporter;
    match reporter.generate_report() {
        Ok(report) => println!("Generated report: {}", report),
        Err(e) => eprintln!("Error generating report: {:?}", e),
    }
}
```

## License

This project is dual-licensed under either the [MIT license](LICENSE-MIT) or the [Apache License, Version 2.0](LICENSE-APACHE), at your option.

## Contributing

Contributions are welcome! Please check out the [repository](https://github.com/klebs6/klebs-general) for details.
