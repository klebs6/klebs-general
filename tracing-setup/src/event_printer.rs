crate::ix!();

#[derive(Debug, Clone)]
pub enum EventPrinter {
    /// Display the entire `tracing::Event` with all debug info.
    FullWithHeader,

    /// A single-line format that can optionally include timestamps/levels.
    LogLineAndContents {
        show_timestamp: bool,
        show_loglevel:  bool,
        show_location:  bool,
    },

    /// Prints only the event's field values.
    JustTheContents,
}

// You can pick whichever default you prefer:
impl Default for EventPrinter {
    fn default() -> Self {
        Self::LogLineAndContents {
            show_timestamp: false,
            show_loglevel:  true,
            show_location:  true,
        }
    }
}
