mod open_options;
#[cfg(not(feature = "filecracker"))]
mod real_file;
#[cfg(not(feature = "filecracker"))]
pub use real_file::File;
#[cfg(feature = "filecracker")]
mod fake_file;
#[cfg(feature = "filecracker")]
pub use fake_file::File;


pub use open_options::OpenOptions;

pub enum CorruptionStyle {
    JournaledFs,
    UnjournaledFs,
    CosmicRay,
    InfantWithMagnet,
}
