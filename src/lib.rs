// There are too many unsafe windows APIs to document everything.
#![allow(clippy::missing_safety_doc)]

pub mod event_loop;
pub mod tank;
pub mod window_winapi;
pub mod bitmap;
pub mod image_processing;

#[derive(Copy, Clone)]
/// x, y coordinate
pub struct Coordinate<T>(pub T, pub T);

#[derive(Copy, Clone)]
/// A size in 2D space
pub struct Size<T>(pub T, pub T);
