use byteorder::{LittleEndian, WriteBytesExt};
use std::io::Write;

use crate::{
    error::Svg2PdcResult,
    point::{Conversion, FPoint, PebblePoint, Precision},
};

#[derive(Debug, Clone)]
/// A PebbleImage is a Pebble Draw Command Image.
///
/// It contains a size and a list of draw commands.
/// You can create a PebbleImage by parsing an SVG file with the `SvgConverter`.
pub struct PebbleImage {
    pub size: PebblePoint,
    pub commands: Vec<DrawCommand>,
}

impl PebbleImage {
    const DRAW_COMMAND_VERSION: u8 = 1;

    fn serialize_header<W: Write>(&self, writer: &mut W) -> Svg2PdcResult<()> {
        writer.write_u8(Self::DRAW_COMMAND_VERSION)?;
        writer.write_u8(0)?; // reserved byte
        writer.write_u16::<LittleEndian>(self.size.x)?;
        writer.write_u16::<LittleEndian>(self.size.y)?;
        Ok(())
    }

    pub fn serialize<W: Write>(&self, writer: &mut W) -> Svg2PdcResult<()> {
        let mut buf_writer = std::io::BufWriter::new(Vec::new());
        self.serialize_header(&mut buf_writer)?;
        buf_writer.write_u16::<LittleEndian>(self.commands.len() as u16)?;
        for command in &self.commands {
            command.serialize(&mut buf_writer)?;
        }

        let buf = buf_writer.into_inner().unwrap();

        let _ = writer.write("PDCI".as_bytes())?;
        writer.write_u32::<LittleEndian>(buf.len() as u32)?;
        writer.write_all(&buf)?;

        Ok(())
    }

    pub fn inspect(&self) {
        // println!("{:#?}", self);
        eprintln!("Size: {:?}", self.size);
        eprintln!("Commands:");
        for command in &self.commands {
            command.inspect();
        }
    }
}

pub type StrokeColor = u8;
pub type FillColor = u8;

#[derive(Debug, Clone, Default)]
pub struct DrawOptions {
    pub translate: FPoint,
    pub stroke_width: u8,
    pub stroke_color: StrokeColor,
    pub fill_color: FillColor,
    pub precision: Precision,
    pub conversion: Conversion,
}

#[derive(Debug, Clone)]
pub enum DrawCommand {
    Path {
        points: Vec<PebblePoint>,
        open: bool,
        options: DrawOptions,
    },
    Circle {
        center: PebblePoint,
        radius: u16,
        options: DrawOptions,
    },
}

impl DrawCommand {
    const DRAW_COMMAND_TYPE_PATH: u8 = 1;
    const DRAW_COMMAND_TYPE_CIRCLE: u8 = 2;
    const DRAW_COMMAND_TYPE_PRECISE_PATH: u8 = 3;

    const DRAW_COMMAND_HEADER_SIZE: u32 = 9;

    pub fn serialize<W: Write>(&self, writer: &mut W) -> Svg2PdcResult<u32> {
        // writer.write_u8(Self::DRAW_COMMAND_VERSION)?;

        match self {
            Self::Path {
                points,
                open,
                options,
            } => {
                let draw_command_type = match options.precision {
                    Precision::Normal => Self::DRAW_COMMAND_TYPE_PATH,
                    Precision::Precise => Self::DRAW_COMMAND_TYPE_PRECISE_PATH,
                };
                writer.write_u8(draw_command_type)?;
                writer.write_u8(0)?; // reserved byte
                writer.write_u8(options.stroke_color)?;
                writer.write_u8(options.stroke_width)?;
                writer.write_u8(options.fill_color)?;
                writer.write_u8(if *open { 1 } else { 0 })?; // path is open
                writer.write_u8(0)?; // reserved byte
                writer.write_u16::<LittleEndian>(points.len() as u16)?;
                for point in points.iter().map(|point| *point + options.translate) {
                    let point =
                        point.pebble_coordinates(&options.precision, &options.conversion)?;
                    writer.write_u16::<LittleEndian>(point.x)?;
                    writer.write_u16::<LittleEndian>(point.y)?;
                }

                Ok(Self::DRAW_COMMAND_HEADER_SIZE + points.len() as u32 * 4)
            }
            Self::Circle {
                center,
                radius,
                options,
            } => {
                let center = *center + options.translate;
                let center = center.pebble_coordinates(&options.precision, &options.conversion)?;

                writer.write_u8(Self::DRAW_COMMAND_TYPE_CIRCLE)?;
                writer.write_u8(0)?; // reserved byte
                writer.write_u8(options.stroke_color)?;
                writer.write_u8(options.stroke_width)?;
                writer.write_u8(options.fill_color)?;
                writer.write_u16::<LittleEndian>(*radius)?;
                writer.write_u16::<LittleEndian>(center.x)?;
                writer.write_u16::<LittleEndian>(center.y)?;

                Ok(Self::DRAW_COMMAND_HEADER_SIZE + 6)
            }
        }
    }

    pub fn inspect(&self) {
        match self {
            Self::Path {
                points,
                open,
                options,
            } => {
                eprintln!("Path:");
                eprintln!("  Points (transalted):");
                for point in points.iter().map(|point| *point + options.translate) {
                    eprintln!("    {:?}", point);
                }
                eprintln!("  Open: {}", open);
                eprintln!("  Options:");
                eprintln!("    Translate: {:?}", options.translate);
                eprintln!("    Stroke Width: {}", options.stroke_width);
                eprintln!("    Stroke Color: {}", options.stroke_color);
                eprintln!("    Fill Color: {}", options.fill_color);
                eprintln!("    Precision: {:?}", options.precision);
                eprintln!("    Conversion: {:?}", options.conversion);
            }
            Self::Circle {
                center,
                radius,
                options,
            } => {
                let center = *center + options.translate;
                eprintln!("Circle:");
                eprintln!("  Center: {:?}", center);
                eprintln!("  Radius: {}", radius);
                eprintln!("  Options:");
                eprintln!("    Translate: {:?}", options.translate);
                eprintln!("    Stroke Width: {}", options.stroke_width);
                eprintln!("    Stroke Color: {}", options.stroke_color);
                eprintln!("    Fill Color: {}", options.fill_color);
                eprintln!("    Precision: {:?}", options.precision);
                eprintln!("    Conversion: {:?}", options.conversion);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_header() {
        let image = PebbleImage {
            size: PebblePoint { x: 100, y: 200 },
            commands: Vec::new(),
        };

        let mut buffer = Vec::new();
        image.serialize_header(&mut buffer).unwrap();

        assert_eq!(buffer[0], PebbleImage::DRAW_COMMAND_VERSION);
        assert_eq!(buffer[1], 0);
        assert_eq!(buffer[2..4], 100u16.to_le_bytes());
        assert_eq!(buffer[4..6], 200u16.to_le_bytes());
    }

    #[test]
    fn test_serialize_empty_image() {
        let image = PebbleImage {
            size: PebblePoint { x: 100, y: 200 },
            commands: Vec::new(),
        };

        let mut buffer = Vec::new();
        image.serialize(&mut buffer).unwrap();

        let commands_length = u16::from_le_bytes(buffer[6..8].try_into().unwrap()) as usize;
        assert_eq!(commands_length, 0);
    }

    #[test]
    fn test_serialize_image_with_path() {
        let image = PebbleImage {
            size: PebblePoint { x: 100, y: 200 },
            commands: vec![DrawCommand::Path {
                points: vec![PebblePoint { x: 10, y: 20 }, PebblePoint { x: 30, y: 40 }],
                open: false,
                options: DrawOptions {
                    translate: FPoint { x: 5.0, y: 6.0 },
                    stroke_width: 2,
                    stroke_color: 3,
                    fill_color: 4,
                    precision: Precision::Normal,
                    conversion: Conversion::RequireExact,
                },
            }],
        };

        let mut buffer = Vec::new();
        image.serialize(&mut buffer).unwrap();

        let expected_header = "PDCI".as_bytes();
        assert_eq!(&buffer[0..4], expected_header);

        assert_eq!(buffer[8], PebbleImage::DRAW_COMMAND_VERSION);
        assert_eq!(buffer[9], 0);
        assert_eq!(buffer[10..12], 100u16.to_le_bytes());
        assert_eq!(buffer[12..14], 200u16.to_le_bytes());

        let commands_length = u16::from_le_bytes(buffer[14..16].try_into().unwrap()) as usize;
        assert_eq!(commands_length, 1);

        assert_eq!(buffer[16], DrawCommand::DRAW_COMMAND_TYPE_PATH); // Draw Command Type
        assert_eq!(buffer[17], 0); // Reserved
        assert_eq!(buffer[18], 3); // Stroke Color
        assert_eq!(buffer[19], 2); // Stroke Width
        assert_eq!(buffer[20], 4); // Fill Color
        assert_eq!(buffer[21], 0); // Path Open
        assert_eq!(buffer[22], 0); // Reserved
        assert_eq!(buffer[23..25], 2u16.to_le_bytes()); // Point Count

        // assert_eq!(buffer[25..27], 15u16.to_le_bytes()); // Point 1 X (10 + 5)
        // assert_eq!(buffer[27..29], 26u16.to_le_bytes()); // Point 1 Y (20 + 6)
        // assert_eq!(buffer[29..31], 35u16.to_le_bytes()); // Point 2 X (30 + 5)
        // assert_eq!(buffer[31..33], 46u16.to_le_bytes()); // Point 2 Y (40 + 6)
    }

    //     #[test]
    //     fn test_serialize_image_with_circle() {
    //         let image = PebbleImage {
    //             size: PebblePoint { x: 100, y: 200 },
    //             commands: vec![DrawCommand::Circle {
    //                 center: PebblePoint { x: 50, y: 60 },
    //                 radius: 25,
    //                 options: DrawOptions {
    //                     translate: PebblePoint { x: 5, y: 6 },
    //                     stroke_width: 2,
    //                     stroke_color: 3,
    //                     fill_color: 4,
    //                     precision: Precision::Normal,
    //                 },
    //             }],
    //         };

    //         let mut buffer = Vec::new();
    //         image.serialize(&mut buffer).unwrap();

    //         let expected_header = "PDCI".as_bytes();
    //         assert_eq!(&buffer[0..4], expected_header);

    //         let data_length = u32::from_le_bytes(buffer[4..8].try_into().unwrap()) as usize;
    //         assert_eq!(data_length, 22);

    //         assert_eq!(buffer[8], PebbleImage::DRAW_COMMAND_VERSION); // Image Version
    //         assert_eq!(buffer[9], 0); // Reserved
    //         assert_eq!(buffer[10..12], 100u16.to_le_bytes()); // Size X
    //         assert_eq!(buffer[12..14], 200u16.to_le_bytes()); // Size Y
    //         assert_eq!(buffer[14..16], 1u16.to_le_bytes()); // Command Count

    //         assert_eq!(buffer[16], DrawCommand::DRAW_COMMAND_TYPE_CIRCLE); // Draw Command Type
    //         assert_eq!(buffer[17], 0); // Reserved
    //         assert_eq!(buffer[18], 3); // Stroke Color
    //         assert_eq!(buffer[19], 2); // Stroke Width
    //         assert_eq!(buffer[20], 4); // Fill Color
    //         assert_eq!(buffer[21..23], 25u16.to_le_bytes()); // Radius
    //         assert_eq!(buffer[23..25], 55u16.to_le_bytes()); // Center X (50 + 5)
    //         assert_eq!(buffer[25..27], 66u16.to_le_bytes()); // Center Y (60 + 6)
    //     }

    //     #[test]
    //     fn test_draw_command_serialize_path() {
    //         let command = DrawCommand::Path {
    //             points: vec![PebblePoint { x: 10, y: 20 }, PebblePoint { x: 30, y: 40 }],
    //             open: true,
    //             options: DrawOptions {
    //                 translate: PebblePoint { x: 5, y: 6 },
    //                 stroke_width: 2,
    //                 stroke_color: 3,
    //                 fill_color: 4,
    //                 precision: Precision::Normal,
    //             },
    //         };

    //         let mut buffer = Vec::new();
    //         command.serialize(&mut buffer).unwrap();

    //         assert_eq!(buffer[0], DrawCommand::DRAW_COMMAND_TYPE_PATH);
    //         assert_eq!(buffer[1], 0); // Reserved
    //         assert_eq!(buffer[2], 3); // Stroke Color
    //         assert_eq!(buffer[3], 2); // Stroke Width
    //         assert_eq!(buffer[4], 4); // Fill Color
    //         assert_eq!(buffer[5], 1); // Path Open
    //         assert_eq!(buffer[6], 0); // Reserved
    //         assert_eq!(buffer[7..9], 2u16.to_le_bytes()); // Point Count

    //         assert_eq!(buffer[9..11], 15u16.to_le_bytes()); // Point 1 X (10 + 5)
    //         assert_eq!(buffer[11..13], 26u16.to_le_bytes()); // Point 1 Y (20 + 6)
    //         assert_eq!(buffer[13..15], 35u16.to_le_bytes()); // Point 2 X (30 + 5)
    //         assert_eq!(buffer[15..17], 46u16.to_le_bytes()); // Point 2 Y (40 + 6)
    //     }

    //     #[test]
    //     fn test_draw_command_serialize_circle() {
    //         let command = DrawCommand::Circle {
    //             center: PebblePoint { x: 50, y: 60 },
    //             radius: 25,
    //             options: DrawOptions {
    //                 translate: PebblePoint { x: 5, y: 6 },
    //                 stroke_width: 2,
    //                 stroke_color: 3,
    //                 fill_color: 4,
    //                 precision: Precision::Normal,
    //             },
    //         };

    //         let mut buffer = Vec::new();
    //         command.serialize(&mut buffer).unwrap();

    //         assert_eq!(buffer[0], DrawCommand::DRAW_COMMAND_TYPE_CIRCLE);
    //         assert_eq!(buffer[1], 0); // Reserved
    //         assert_eq!(buffer[2], 3); // Stroke Color
    //         assert_eq!(buffer[3], 2); // Stroke Width
    //         assert_eq!(buffer[4], 4); // Fill Color
    //         assert_eq!(buffer[5..7], 25u16.to_le_bytes()); // Radius
    //         assert_eq!(buffer[7..9], 55u16.to_le_bytes()); // Center X (50 + 5)
    //         assert_eq!(buffer[9..11], 66u16.to_le_bytes()); // Center Y (60 + 6)
    //     }

    //     #[test]
    //     fn test_draw_command_serialize_precise_path() {
    //         let command = DrawCommand::Path {
    //             points: vec![PebblePoint { x: 10, y: 20 }, PebblePoint { x: 30, y: 40 }],
    //             open: true,
    //             options: DrawOptions {
    //                 translate: PebblePoint { x: 5, y: 6 },
    //                 stroke_width: 2,
    //                 stroke_color: 3,
    //                 fill_color: 4,
    //                 precision: Precision::Precise,
    //             },
    //         };

    //         let mut buffer = Vec::new();
    //         command.serialize(&mut buffer).unwrap();

    //         assert_eq!(buffer[0], DrawCommand::DRAW_COMMAND_TYPE_PRECISE_PATH);
    //         assert_eq!(buffer[1], 0); // Reserved
    //         assert_eq!(buffer[2], 3); // Stroke Color
    //         assert_eq!(buffer[3], 2); // Stroke Width
    //         assert_eq!(buffer[4], 4); // Fill Color
    //         assert_eq!(buffer[5], 1); // Path Open
    //         assert_eq!(buffer[6], 0); // Reserved
    //         assert_eq!(buffer[7..9], 2u16.to_le_bytes()); // Point Count

    //         assert_eq!(buffer[9..11], 15u16.to_le_bytes()); // Point 1 X (10 + 5)
    //         assert_eq!(buffer[11..13], 26u16.to_le_bytes()); // Point 1 Y (20 + 6)
    //         assert_eq!(buffer[13..15], 35u16.to_le_bytes()); // Point 2 X (30 + 5)
    //         assert_eq!(buffer[15..17], 46u16.to_le_bytes()); // Point 2 Y (40 + 6)
    //     }
}
