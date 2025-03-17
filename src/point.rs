use std::ops::{Add, Div, Mul, Sub};

use crate::error::{Svg2PdcError, Svg2PdcResult};

#[derive(Debug, Clone, Copy, Default)]
pub enum Precision {
    #[default]
    Normal,
    Precise,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Conversion {
    ConvertNoWarn,
    ConvertWarn,
    #[default]
    RequireExact,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct FPoint {
    pub x: f32,
    pub y: f32,
}

impl FPoint {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn round(&self) -> Self {
        // Add f32::EPSILON was carried over from the original code in python
        Self {
            x: (self.x + f32::EPSILON).round(),
            y: (self.y + f32::EPSILON).round(),
        }
    }

    pub fn find_nearest_valid(&self, precision: &Precision) -> Self {
        let constant = match precision {
            Precision::Normal => 2.0,
            Precision::Precise => 8.0,
        };
        (*self * constant).round() / constant
    }

    pub fn pebble_coordinates(
        &self,
        precision: &Precision,
        conversion: &Conversion,
    ) -> Svg2PdcResult<PebblePoint> {
        let nearest_valid = (*self).find_nearest_valid(precision);
        let point = if self != &nearest_valid {
            match conversion {
                Conversion::ConvertNoWarn => nearest_valid,
                Conversion::ConvertWarn => {
                    eprintln!(
                        "Warning: Point {:?} is not a valid pebble coordinate. Using nearest valid point {:?}",
                        self, nearest_valid
                    );
                    nearest_valid
                }
                Conversion::RequireExact => {
                    return Err(Svg2PdcError::InvalidPoint {
                        point: *self,
                        nearest_valid,
                    });
                }
            }
        } else {
            *self
        };
        let translated = point + FPoint::new(-0.5, -0.5);

        let translated = translated.round();
        let translated = match precision {
            Precision::Normal => translated,
            Precision::Precise => translated * 8.0,
        };
        Ok(PebblePoint {
            x: translated.x as u16,
            y: translated.y as u16,
        })
    }
}

impl Add for FPoint {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for FPoint {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<f32> for FPoint {
    type Output = Self;

    fn mul(self, other: f32) -> Self {
        Self {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl Div<f32> for FPoint {
    type Output = Self;

    fn div(self, other: f32) -> Self {
        Self {
            x: self.x / other,
            y: self.y / other,
        }
    }
}
#[derive(Debug, Clone, Copy, Default)]
pub struct PebblePoint {
    pub x: u16,
    pub y: u16,
}

impl From<PebblePoint> for FPoint {
    fn from(pebble_coordinates: PebblePoint) -> Self {
        Self {
            x: pebble_coordinates.x as f32,
            y: pebble_coordinates.y as f32,
        }
    }
}

impl Add for PebblePoint {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Add<FPoint> for PebblePoint {
    type Output = FPoint;

    fn add(self, other: FPoint) -> FPoint {
        FPoint {
            x: self.x as f32 + other.x,
            y: self.y as f32 + other.y,
        }
    }
}
