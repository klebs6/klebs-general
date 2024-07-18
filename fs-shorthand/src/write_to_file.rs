crate::ix!();

pub fn write_to_file<T: Display>(filename: &str, items: &[T]) -> io::Result<()> {
    let file = File::create(filename)?;
    let mut writer = std::io::BufWriter::new(file);

    for item in items {
        writeln!(writer, "{}", item)?;
    }

    writer.flush()  // Ensure all data is written to the file before closing
}
