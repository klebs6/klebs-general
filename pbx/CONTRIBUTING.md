# Contributing to pbx

Thank you for your interest in contributing to `pbx`! We welcome all kinds of contributions, including bug reports, feature requests, and code improvements. To make the contribution process smooth, please follow the guidelines below.

## Getting Started

### Fork the Repository

1. Fork the repository on GitHub.
2. Clone your forked repository to your local machine:

    ```sh
    git clone https://github.com/yourusername/pbx.git
    cd pbx
    ```

3. Add the upstream repository:

    ```sh
    git remote add upstream https://github.com/originalusername/pbx.git
    ```

### Set Up the Development Environment

1. Ensure you have [Rust](https://www.rust-lang.org/tools/install) installed.
2. Install the necessary tools:

    ```sh
    rustup component add clippy
    rustup component add rustfmt
    ```

3. Build the project:

    ```sh
    cargo build
    ```

4. Run tests to make sure everything is set up correctly:

    ```sh
    cargo test
    ```

## Making Changes

1. Create a new branch for your changes:

    ```sh
    git checkout -b my-feature-branch
    ```

2. Make your changes, ensuring that you follow the Rust coding conventions and include comments where appropriate.
3. Format your code:

    ```sh
    cargo fmt
    ```

4. Lint your code:

    ```sh
    cargo clippy
    ```

5. Add tests for your changes and ensure all tests pass:

    ```sh
    cargo test
    ```

6. Commit your changes with a clear and concise commit message:

    ```sh
    git commit -am "Add feature X"
    ```

## Submitting Changes

1. Push your changes to your forked repository:

    ```sh
    git push origin my-feature-branch
    ```

2. Open a pull request on the original repository.
3. Provide a clear description of your changes in the pull request, including the motivation and context for the change.
4. Wait for feedback and be ready to make necessary adjustments based on comments.

## Code of Conduct

We are committed to providing a welcoming and inclusive environment for all contributors. Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md).

## Reporting Issues

If you encounter any issues or have suggestions for improvements, please open an issue on GitHub. Provide as much detail as possible to help us understand and resolve the issue.

## License

By contributing to `pbx`, you agree that your contributions will be licensed under the MIT License.

Thank you for your contributions!

