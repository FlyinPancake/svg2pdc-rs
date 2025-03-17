use crate::point::FPoint;

#[derive(Debug, thiserror::Error)]
pub enum Svg2PdcError {
    #[error("Invalid point. Point: {point:?}, nearest valid: {nearest_valid:?}")]
    InvalidPoint {
        point: FPoint,
        nearest_valid: FPoint,
    },
    #[error("IO error: `{0}`")]
    Io(#[from] std::io::Error),
    #[error("XML error: `{0}`")]
    XmlError(#[from] roxmltree::Error),
    #[error("Invalid viewBox: `{0}`")]
    InvalidViewBox(#[from] svgtypes::ViewBoxError),
    #[error("Invalid polyline: `{0}`")]
    InvalidPolyline(String),
    #[error("SvgTypes error: `{0}`")]
    SvgTypesError(#[from] svgtypes::Error),
    #[error("Invalid color string: `{0}`")]
    InvalidColor(String),
    #[error("Unsupported circle format")]
    UnsupportedCircle,
    #[error("Parse Error {0}")]
    ParseError(String),
    #[error("Unsupported Operation `{0}`")]
    UnsupportedOperation(String),
}

pub type Svg2PdcResult<T> = Result<T, Svg2PdcError>;
