pub mod color;
pub mod error;
pub mod image;
pub mod point;
pub mod svg_converter;

pub mod prelude {
    pub use crate::color::{Color, PebbleColor, TruncateColor};
    pub use crate::error::{Svg2PdcError, Svg2PdcResult};
    pub use crate::image::{DrawCommand, DrawOptions, FillColor, PebbleImage, StrokeColor};
    pub use crate::point::{FPoint, Precision};
    pub use crate::svg_converter::SvgConverter;
}
