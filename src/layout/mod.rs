pub mod floating;

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutType {
    Floating,
    Tiled,
}