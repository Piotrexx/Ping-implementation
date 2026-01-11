#[cfg(unix)]
mod unix;

#[cfg(windows)]
mod win;

#[cfg(unix)]
pub use unix::*;

#[cfg(windows)]
pub use win::*;