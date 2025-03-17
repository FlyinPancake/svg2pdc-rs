use crate::error::{Svg2PdcError, Svg2PdcResult};

#[derive(Debug, Clone, Copy)]
pub enum TruncateColor {
    Truncate,
    Keep,
}

/// A color in the format of a 32-bit RGBA color.
///
/// The color is stored as 4 bytes
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Create a new color from a hex string.
    ///
    /// The hex string can be in the format `#RRGGBB` or `#RRGGBBAA`.
    ///
    /// ```rust
    /// use svg2pdc::color::Color;
    ///
    /// let red = Color::try_from_hex("#ff0000").unwrap();
    /// let red_with_alpha = Color::try_from_hex("#ff0000ff").unwrap();
    /// assert_eq!(red, red_with_alpha);
    /// assert_eq!(red.a, 255);
    /// assert_eq!(red.r, 255);
    /// assert_eq!(red.g, 0);
    /// assert_eq!(red.b, 0);
    ///
    /// let green_1 = Color::try_from_hex("00ff00f0").unwrap();
    /// assert_eq!(green_1.a, 240);
    /// let green_2 = Color::try_from_hex("00ff00").unwrap().with_opacity(0xf0);
    /// assert_eq!(green_2.a, 240);
    /// assert_eq!(green_1, green_2);
    /// ```
    pub fn try_from_hex(hex: &str) -> Svg2PdcResult<Self> {
        let hex = hex.trim_start_matches('#');
        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|_| Svg2PdcError::InvalidColor(hex.to_string()))?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| Svg2PdcError::InvalidColor(hex.to_string()))?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|_| Svg2PdcError::InvalidColor(hex.to_string()))?;
        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16).unwrap_or(255)
        } else {
            255
        };
        Ok(Self { r, g, b, a })
    }

    /// Modify the opacity of a color.
    ///
    /// ```rust
    /// use svg2pdc::color::Color;
    ///
    /// let red = Color::try_from_hex("#ff0000").unwrap();
    /// let red_with_alpha = red.with_opacity(128);
    /// assert_eq!(red_with_alpha.a, 128);
    /// assert_eq!(red.with_opacity(128), red_with_alpha);
    ///
    /// let red_with_alpha = red.with_opacity(255);
    /// assert_eq!(red_with_alpha.a, 255);
    /// ```
    pub fn with_opacity(&self, opacity: u8) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a: opacity,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
/// A color in Pebble's format.
///
/// The color is stored in a single byte, with the following format:
///
/// ```text
/// 0b0000_0000
///   |||| ||||
///   |||| ||++-- Blue  (0-31)
///   |||| ++---- Green (0-31)
///   ||++------- Red   (0-31)
///   ++--------- Alpha (0-31)
/// ```
pub struct PebbleColor(u8);

impl PebbleColor {
    pub const fn nothing() -> Self {
        Self(0)
    }

    /// Pack a color into a PebbleColor.
    ///
    /// Don't use this function directly, use `from_color_with_convert` or `from_color_with_truncate` instead.
    const fn from_color(Color { r, g, b, a }: Color) -> Self {
        Self((a << 6) | (r << 4) | (g << 2) | b)
    }

    /// Create a new PebbleColor from a Color.
    ///
    /// Will truncate the color with rounding.
    pub const fn from_color_with_truncate(Color { r, g, b, a }: Color) -> Self {
        let a = (a / 85) * 85;

        if a == 0 {
            return Self(0);
        }

        let r = (r / 85) * 85;
        let g = (g / 85) * 85;
        let b = (b / 85) * 85;

        Self::from_color(Color { r, g, b, a })
    }

    /// Create a new PebbleColor from a Color.
    ///
    /// Will convert the color to the nearest color in the Pebble palette.
    ///
    /// ```rust
    /// use svg2pdc::color::{PebbleColor, Color};
    ///
    /// let white = Color::try_from_hex("#ffffff").unwrap();
    /// let pebble_white = PebbleColor::from_color_with_convert(white);
    ///
    /// assert_eq!(pebble_white.get_r(), 3);
    /// assert_eq!(pebble_white.get_g(), 3);
    /// assert_eq!(pebble_white.get_b(), 3);
    /// assert_eq!(pebble_white.get_a(), 3);
    /// ```
    pub const fn from_color_with_convert(Color { r, g, b, a }: Color) -> Self {
        let a = (((a as f32 + 42_f32) / 85_f32) * 85_f32) as u8;
        if a == 0 {
            return Self(0);
        }

        let r = (((r as f32 + 42_f32) / 85_f32) * 85_f32) as u8;
        let g = (((g as f32 + 42_f32) / 85_f32) * 85_f32) as u8;
        let b = (((b as f32 + 42_f32) / 85_f32) * 85_f32) as u8;

        Self::from_color(Color { r, g, b, a })
    }

    /// Get the alpha component of the color.
    ///
    /// The alpha component is stored as 2 bits.
    ///
    /// ```rust
    /// use svg2pdc::color::{PebbleColor, Color};
    ///
    /// let red = Color::try_from_hex("#ff0000ff").unwrap();
    ///
    /// let pebble_red = PebbleColor::from_color_with_truncate(red);
    /// assert_eq!(pebble_red.get_a(), 3);
    /// ```
    pub const fn get_a(&self) -> u8 {
        (self.0 & 0b1100_0000) >> 6
    }

    /// Get the red component of the color.
    ///
    /// The red component is stored as 2 bits.
    ///
    /// ```rust
    /// use svg2pdc::color::{PebbleColor, Color};
    ///
    /// let red = Color::try_from_hex("#ff0000").unwrap();
    /// let red = PebbleColor::from_color_with_truncate(red);
    ///
    /// assert_eq!(red.get_r(), 3);
    /// ```
    pub const fn get_r(&self) -> u8 {
        (self.0 & 0b0011_0000) >> 4
    }

    /// Get the green component of the color.
    ///
    /// The green component is stored as 2 bits.
    ///
    /// ```rust
    /// use svg2pdc::color::{PebbleColor, Color};
    ///
    /// let green = Color::try_from_hex("#00ff00").unwrap();
    /// let green = PebbleColor::from_color_with_truncate(green);
    ///
    /// assert_eq!(green.get_g(), 3);
    /// ```
    pub const fn get_g(&self) -> u8 {
        (self.0 & 0b0000_1100) >> 2
    }

    /// Get the blue component of the color.
    ///
    /// The blue component is stored as 2 bits.
    ///
    /// ```rust
    /// use svg2pdc::color::{PebbleColor, Color};
    ///
    /// let blue = Color::try_from_hex("#0000ff").unwrap();
    /// let blue = PebbleColor::from_color_with_truncate(blue);
    ///
    /// assert_eq!(blue.get_b(), 3);
    /// ```
    pub const fn get_b(&self) -> u8 {
        self.0 & 0b0000_0011
    }

    /// Check if the color is black.
    ///
    /// A color is considered black if all of its components are 0.
    ///
    /// ```rust
    /// use svg2pdc::color::{PebbleColor, Color};
    ///
    /// let red = Color::try_from_hex("#ff0000").unwrap().with_opacity(255);
    /// assert!(!PebbleColor::from_color_with_truncate(red).is_black());
    ///
    /// let black = Color::try_from_hex("#00000000").unwrap();
    /// assert!(PebbleColor::from_color_with_truncate(black).is_black());
    /// ```
    pub const fn is_black(&self) -> bool {
        self.0 & 0b0011_1111 == 0
    }

    /// Get the bitdepth of a color palette.
    ///
    /// Not sure if this is needed for anything, ported for completion's sake.
    pub const fn num_colors_to_bitdepth(num_colors: u32) -> Option<u8> {
        match num_colors {
            1..=2 => Some(1),
            3..=4 => Some(2),
            5..=16 => Some(4),
            17..=256 => Some(8),
            _ => None,
        }
    }

    /// Get the inner value of the PebbleColor.
    ///
    /// Used for serialization.
    ///
    /// ```rust
    /// use svg2pdc::color::{PebbleColor, Color};
    ///
    /// let red = Color::try_from_hex("#ff0000").unwrap().with_opacity(255);
    ///
    /// let pebble_red = PebbleColor::from_color_with_truncate(red);
    ///
    /// assert_eq!(pebble_red.inner(), 192 + 48;
    /// ```
    pub const fn inner(&self) -> u8 {
        self.0
    }

    // fn truncate_to_pebble_palette
}
