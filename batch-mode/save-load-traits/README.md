# save-load-traits

The `save-load-traits` crate provides traits for saving and loading objects asynchronously to and from files and directories. It also includes a blanket implementation for loading objects from a directory using the `LoadFromFile` trait.

## Key Features
- **SaveToFile Trait**: Defines a trait for saving objects to a file asynchronously.
- **LoadFromFile Trait**: Defines a trait for loading objects from a file asynchronously.
- **LoadFromDirectory Trait**: Extends the functionality of loading from a directory, supporting automatic loading of files from a directory based on the `LoadFromFile` trait.

## Traits and Implementations

### SaveToFile Trait
The `SaveToFile` trait allows objects to be saved asynchronously to a file.

```rust
#[async_trait]
pub trait SaveToFile {
    type Error;

    async fn save_to_file(&self, filename: impl AsRef<Path> + Send) -> Result<(), Self::Error>;
}
```

### LoadFromFile Trait
The `LoadFromFile` trait allows objects to be loaded asynchronously from a file.

```rust
#[async_trait]
pub trait LoadFromFile: Sized {
    type Error;

    async fn load_from_file(filename: impl AsRef<Path> + Send) -> Result<Self, Self::Error>;
}
```

### LoadFromDirectory Trait
The `LoadFromDirectory` trait is designed for asynchronously loading objects from a directory. It provides a blanket implementation for any type that implements `LoadFromFile`.

```rust
#[async_trait]
pub trait LoadFromDirectory: Sized {
    type Error;

    async fn load_from_directory(
        dir: impl AsRef<Path> + Send,
    ) -> Result<Vec<Self>, Self::Error>;
}
```

## Error Handling
This crate uses custom error types to handle I/O and JSON parsing errors:

```rust
error_tree! {
    pub enum SaveLoadError {
        IoError(std::io::Error),
        JsonParseError(JsonParseError),
        InvalidDirectory { dir: PathBuf },
    }
}
```

## Usage

### Save an Object to a File

To save an object to a file asynchronously, implement the `SaveToFile` trait for the type:

```rust
let obj = YourStruct {};
obj.save_to_file("file.json").await?;
```

### Load an Object from a File

To load an object from a file asynchronously, implement the `LoadFromFile` trait for the type:

```rust
let obj = YourStruct::load_from_file("file.json").await?;
```

### Load Objects from a Directory

To load multiple objects from a directory, use the `LoadFromDirectory` trait:

```rust
let objects = YourStruct::load_from_directory("directory_path").await?;
```

## License
This crate is licensed under the MIT License. See LICENSE for details.
