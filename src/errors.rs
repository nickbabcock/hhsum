use std::io;

#[derive(Debug, Fail)]
pub enum HashError {
    #[fail(display = "unable to retrieve metadata for: {}", file)]
    Metadata {
        file: String,
        #[cause]
        error: io::Error,
    },

    #[fail(display = "unable to open file: {}", file)]
    File {
        file: String,
        #[cause]
        error: io::Error,
    },

    #[fail(display = "unable to memmap file: {}", file)]
    Memmap {
        file: String,
        #[cause]
        error: io::Error,
    },

    #[fail(display = "unable to read file: {}", file)]
    Read {
        file: String,
        #[cause]
        error: io::Error,
    },
}
