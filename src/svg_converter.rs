use std::{collections::HashMap, num::ParseFloatError, str::FromStr};

use roxmltree::{Document, Node};
use svgtypes::{PathSegment, TransformListToken, ViewBox};

use crate::{
    color::{Color, PebbleColor, TruncateColor},
    error::{Svg2PdcError, Svg2PdcResult},
    image::{DrawCommand, DrawOptions, PebbleImage},
    point::{Conversion, FPoint, Precision},
};

#[derive(Debug, Clone, Default)]
struct GroupOptions {
    pub opacity: Option<f64>,
    pub fill_color: Option<String>,
    pub fill_opacity: Option<f64>,
    pub stroke_color: Option<String>,
    pub stroke_opacity: Option<f64>,
    pub stroke_width: Option<u8>,
}

pub struct SvgConverter {
    pub precision: Precision,
}

impl SvgConverter {
    pub fn new(precision: Precision) -> Self {
        Self { precision }
    }
    fn get_viewbox(document: &Document<'_>) -> Svg2PdcResult<svgtypes::ViewBox> {
        let root = document.root_element();
        let view_box = match root.attribute("viewBox") {
            Some(view_box) => ViewBox::from_str(view_box)?,
            None => ViewBox {
                x: 0.0,
                y: 0.0,
                w: root.attribute("width").unwrap_or("0").parse().unwrap(),
                h: root.attribute("height").unwrap_or("0").parse().unwrap(),
            },
        };
        Ok(view_box)
    }

    fn get_commands(
        &self,
        translation: &FPoint,
        truncate_color: &TruncateColor,
        group_options: &GroupOptions,
        conversion: &Conversion,
        node: Node<'_, '_>,
    ) -> Svg2PdcResult<Vec<DrawCommand>> {
        let mut commands = Vec::new();

        for child in node.children() {
            let display = child.attribute("display");
            if let Some("none") = display {
                continue;
            }
            let tag = child.tag_name().name();

            match tag {
                "layer" | "g" => {
                    if tag == "g" {
                        let subgroup_options = GroupOptions {
                            opacity: child
                                .attribute("opacity")
                                .map(|opacity| opacity.parse().unwrap()),
                            fill_color: child.attribute("fill").map(|fill| fill.to_string()),
                            fill_opacity: child
                                .attribute("fill-opacity")
                                .map(|fill_opacity| fill_opacity.parse().unwrap()),
                            stroke_color: child
                                .attribute("stroke")
                                .map(|stroke| stroke.to_string()),
                            stroke_opacity: child
                                .attribute("stroke-opacity")
                                .map(|stroke_opacity| stroke_opacity.parse().unwrap()),
                            stroke_width: child.attribute("stroke-width").map(|stroke_width| {
                                stroke_width
                                    .chars()
                                    .filter(|c| "1234567890.".contains(*c))
                                    .collect::<String>()
                                    .parse()
                                    .unwrap()
                            }),
                        };

                        let translate = self.get_child_translation(child)?;

                        commands.extend(self.get_commands(
                            &(translate + *translation),
                            truncate_color,
                            &subgroup_options,
                            conversion,
                            child,
                        )?);
                    }
                }
                _ => {
                    let translate = self.get_child_translation(child)? + *translation;
                    let command = self.create_command(
                        &translate,
                        truncate_color,
                        group_options,
                        conversion,
                        child,
                    )?;
                    if let Some(command) = command {
                        commands.push(command);
                    }
                }
            }
        }
        Ok(commands)
    }

    fn create_command(
        &self,
        translation: &FPoint,
        truncate_color: &TruncateColor,
        group_options: &GroupOptions,
        conversion: &Conversion,
        node: Node<'_, '_>,
    ) -> Svg2PdcResult<Option<DrawCommand>> {
        let mut style: HashMap<String, String> = node
            .attribute("style")
            .unwrap_or("")
            .split(';')
            .map(|style| {
                let mut parts = style.split(':');
                let key = parts.next().unwrap_or("").trim();
                let value = parts.next().unwrap_or("").trim();
                (key.to_string(), value.to_string())
            })
            .collect();
        let attributes: HashMap<String, String> = node
            .attributes()
            .map(|attr| {
                (
                    attr.name().to_string().to_lowercase(),
                    attr.value().to_string().to_lowercase(),
                )
            })
            .collect();

        style.extend(attributes);

        let stroke = style.get("stroke").or(group_options.stroke_color.as_ref());
        let stroke_width = style
            .get("stroke-width")
            .map_or(group_options.stroke_width, |width| {
                width.parse::<f32>().map(|n| n as u8).ok()
            });

        let fill = style.get("fill").or(group_options.fill_color.as_ref());

        let opacity = style
            .get("opacity")
            .map_or(group_options.opacity, |opacity| {
                Some(opacity.parse().unwrap())
            })
            .unwrap_or(1.0) as f32;
        let stroke_opacity = style
            .get("stroke-opacity")
            .map_or(group_options.stroke_opacity, |opacity| {
                Some(opacity.parse().unwrap())
            })
            .unwrap_or(1.0) as f32;

        let fill_opacity = style
            .get("fill-opacity")
            .map_or(group_options.fill_opacity, |opacity| {
                Some(opacity.parse().unwrap())
            })
            .unwrap_or(1.0) as f32;

        let stroke_color = stroke
            .map(|color| Color::try_from_hex(color).unwrap_or_default())
            .unwrap_or_default()
            .with_opacity((opacity * stroke_opacity * 255.0) as u8);
        let stroke_color = match truncate_color {
            TruncateColor::Truncate => PebbleColor::from_color_with_truncate(stroke_color),
            TruncateColor::Keep => PebbleColor::from_color_with_convert(stroke_color),
        };

        let fill_color = fill
            .map(|color| Color::try_from_hex(color).unwrap_or_default())
            .unwrap_or_default()
            .with_opacity((opacity * fill_opacity * 255.0) as u8);
        let fill_color = match truncate_color {
            TruncateColor::Truncate => PebbleColor::from_color_with_truncate(fill_color),
            TruncateColor::Keep => PebbleColor::from_color_with_convert(fill_color),
        };

        // This is a pebble caveat, if the fill color is black, it will be treated as transparent
        let fill_color = if fill_color.is_black() {
            PebbleColor::nothing()
        } else {
            fill_color
        };

        // if stroke_color == PebbleColor::nothing() && fill_color == PebbleColor::nothing() {
        //     return Ok(None);
        // }

        let stroke_width = stroke_width.unwrap_or(1);

        let stroke_width = if stroke_color == PebbleColor::nothing() {
            0
        } else {
            stroke_width
        };

        let stroke_color = if stroke_width == 0 {
            PebbleColor::nothing()
        } else {
            stroke_color
        };

        let tag = node.tag_name().name();

        let options = DrawOptions {
            translate: *translation,
            stroke_width,
            stroke_color: stroke_color.inner(),
            fill_color: fill_color.inner(),
            precision: self.precision,
            conversion: *conversion,
        };

        match tag {
            "path" => Ok(Some(self.parse_path(node, options)?)),
            "circle" => Ok(Some(self.parse_circle(node, options)?)),
            "polyline" => Ok(Some(self.parse_polyline(node, options)?)),
            "polygon" => Ok(Some(self.parse_polygon(node, options)?)),
            "line" => Ok(Some(self.parse_line(node, options)?)),
            "rect" => Ok(Some(self.parse_rect(node, options)?)),
            "g" | "layer" => unreachable!(),
            "" => Ok(None), // skip empty nodes
            // tag => Err(Svg2PdcError::UnsupportedTag(tag.to_string())),
            tag => {
                eprintln!("Skipping unsupported tag: {}", tag);
                Ok(None)
            }
        }
    }

    fn parse_path(&self, node: Node<'_, '_>, options: DrawOptions) -> Svg2PdcResult<DrawCommand> {
        let d = node.attribute("d").unwrap_or("");
        let path = svgtypes::PathParser::from(d);
        let path_segments: Result<Vec<_>, svgtypes::Error> = path.collect();
        let path_segments = path_segments?;

        let mut points = Vec::new();
        let mut current_point = FPoint::default();

        for segment in path_segments {
            match segment {
                PathSegment::MoveTo { abs, x, y }
                | PathSegment::LineTo { abs, x, y }
                | PathSegment::SmoothCurveTo { abs, x, y, .. }
                | PathSegment::CurveTo { abs, x, y, .. }
                | PathSegment::Quadratic { abs, x, y, .. }
                | PathSegment::SmoothQuadratic { abs, x, y }
                | PathSegment::EllipticalArc { abs, x, y, .. } => {
                    let point = match abs {
                        true => FPoint::new(x as f32, y as f32),
                        false => FPoint::new(x as f32, y as f32) + current_point,
                    };
                    points.push(point);
                    current_point = point;
                }

                PathSegment::HorizontalLineTo { abs, x } => {
                    let point = match abs {
                        true => FPoint::new(x as f32, current_point.y),
                        false => FPoint::new(x as f32, current_point.y) + current_point,
                    };
                    points.push(point);
                    current_point = point;
                }
                PathSegment::VerticalLineTo { abs, y } => {
                    let point = match abs {
                        true => FPoint::new(current_point.x, y as f32),
                        false => FPoint::new(current_point.x, y as f32) + current_point,
                    };
                    points.push(point);
                    current_point = point;
                }
                PathSegment::ClosePath { .. } => {
                    if current_point != *points.first().unwrap_or(&FPoint::default()) {
                        points.push(points[0]);
                    }
                }
            }
        }

        // Chopping decicmal points as instead of rounding them to maintain binary compatibility with the original implementation
        // TODO: introduce a new option to allow rounding
        let mut points = points
            .iter()
            .map(|point| FPoint::new(point.x.floor(), point.y.floor()))
            .collect::<Vec<_>>();

        let first = *points.first().unwrap_or(&FPoint::default());
        let last = *points.last().unwrap_or(&FPoint::default());

        let open = first != last;

        if !open {
            points.pop();
        }

        let points = points
            .iter()
            .map(|point| point.pebble_coordinates(&options.precision, &options.conversion))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(DrawCommand::Path {
            points,
            open,
            options,
        })
    }

    fn parse_circle(&self, node: Node<'_, '_>, options: DrawOptions) -> Svg2PdcResult<DrawCommand> {
        let cx = node
            .attribute("cx")
            .ok_or(Svg2PdcError::UnsupportedCircle)?
            .parse::<f32>()
            .map_err(|_| Svg2PdcError::UnsupportedCircle)?;

        let cy = node
            .attribute("cy")
            .ok_or(Svg2PdcError::UnsupportedCircle)?
            .parse::<f32>()
            .map_err(|_| Svg2PdcError::UnsupportedCircle)?;

        let radius = match node.attribute("r") {
            Some(r) => Some(r),
            None => node.attribute("z"),
        }
        .ok_or(Svg2PdcError::UnsupportedCircle)?
        .parse::<f32>()
        .map_err(|_| Svg2PdcError::UnsupportedCircle)?;
        // Circle does not support precise coordinates
        let center =
            FPoint::new(cx, cy).pebble_coordinates(&Precision::Normal, &options.conversion)?;

        Ok(DrawCommand::Circle {
            center,
            radius: radius as u16,
            options,
        })
    }

    fn parse_polyline(
        &self,
        node: Node<'_, '_>,
        options: DrawOptions,
    ) -> Svg2PdcResult<DrawCommand> {
        let points = node
            .attribute("points")
            .ok_or(Svg2PdcError::InvalidPolyline(format!("{node:?}")))?;
        let points = self.get_points_from_str(points)?;

        let points = points
            .iter()
            .map(|point| point.pebble_coordinates(&options.precision, &options.conversion))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(DrawCommand::Path {
            points,
            open: true,
            options,
        })
    }

    fn parse_polygon(
        &self,
        node: Node<'_, '_>,
        options: DrawOptions,
    ) -> Svg2PdcResult<DrawCommand> {
        let points = node
            .attribute("points")
            .ok_or(Svg2PdcError::InvalidPolyline(format!("{node:?}")))?;
        let points = self.get_points_from_str(points)?;

        let points = points
            .iter()
            .map(|point| point.pebble_coordinates(&options.precision, &options.conversion))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(DrawCommand::Path {
            points,
            open: false,
            options,
        })
    }

    fn parse_line(&self, node: Node<'_, '_>, options: DrawOptions) -> Svg2PdcResult<DrawCommand> {
        let x1 = node
            .attribute("x1")
            .ok_or(Svg2PdcError::InvalidPolyline(format!("{node:?}")))?;
        let x1 = x1
            .parse::<f32>()
            .map_err(|_| Svg2PdcError::InvalidPolyline(format!("{node:?}")))?;

        let y1 = node
            .attribute("y1")
            .ok_or(Svg2PdcError::InvalidPolyline(format!("{node:?}")))?;
        let y1 = y1
            .parse::<f32>()
            .map_err(|_| Svg2PdcError::InvalidPolyline(format!("{node:?}")))?;

        let x2 = node
            .attribute("x2")
            .ok_or(Svg2PdcError::InvalidPolyline(format!("{node:?}")))?;
        let x2 = x2
            .parse::<f32>()
            .map_err(|_| Svg2PdcError::InvalidPolyline(format!("{node:?}")))?;

        let y2 = node
            .attribute("y2")
            .ok_or(Svg2PdcError::InvalidPolyline(format!("{node:?}")))?;
        let y2 = y2
            .parse::<f32>()
            .map_err(|_| Svg2PdcError::InvalidPolyline(format!("{node:?}")))?;

        let points = vec![
            FPoint::new(x1, y1).pebble_coordinates(&options.precision, &options.conversion)?,
            FPoint::new(x2, y2).pebble_coordinates(&options.precision, &options.conversion)?,
        ];

        Ok(DrawCommand::Path {
            points,
            open: true,
            options,
        })
    }

    fn parse_rect(&self, node: Node<'_, '_>, options: DrawOptions) -> Svg2PdcResult<DrawCommand> {
        let x = node
            .attribute("x")
            .ok_or(Svg2PdcError::InvalidPolyline(format!("{node:?}")))?;
        let x = x
            .parse::<f32>()
            .map_err(|_| Svg2PdcError::InvalidPolyline(format!("{node:?}")))?;

        let y = node
            .attribute("y")
            .ok_or(Svg2PdcError::InvalidPolyline(format!("{node:?}")))?;
        let y = y
            .parse::<f32>()
            .map_err(|_| Svg2PdcError::InvalidPolyline(format!("{node:?}")))?;

        let width = node
            .attribute("width")
            .ok_or(Svg2PdcError::InvalidPolyline(format!("{node:?}")))?;
        let width = width
            .parse::<f32>()
            .map_err(|_| Svg2PdcError::InvalidPolyline(format!("{node:?}")))?;

        let height = node
            .attribute("height")
            .ok_or(Svg2PdcError::InvalidPolyline(format!("{node:?}")))?;
        let height = height
            .parse::<f32>()
            .map_err(|_| Svg2PdcError::InvalidPolyline(format!("{node:?}")))?;

        let points = vec![
            FPoint::new(x, y).pebble_coordinates(&options.precision, &options.conversion)?,
            FPoint::new(x + width, y)
                .pebble_coordinates(&options.precision, &options.conversion)?,
            FPoint::new(x + width, y + height)
                .pebble_coordinates(&options.precision, &options.conversion)?,
            FPoint::new(x, y + height)
                .pebble_coordinates(&options.precision, &options.conversion)?,
        ];

        Ok(DrawCommand::Path {
            points,
            open: false,
            options,
        })
    }

    fn get_points_from_str(&self, points: &str) -> Svg2PdcResult<Vec<FPoint>> {
        let points_list: Result<Vec<FPoint>, ParseFloatError> = points
            .split_whitespace()
            .map(|chunk| {
                let mut parts = chunk.split(',');
                let x = parts.next().unwrap_or("").parse()?;
                let y = parts.next().unwrap_or("").parse()?;
                Ok(FPoint::new(x, y))
            })
            .collect();
        let points = points_list.map_err(|_| Svg2PdcError::ParseError(points.to_string()))?;
        Ok(points)
    }

    fn get_child_translation(&self, child: Node<'_, '_>) -> Result<FPoint, Svg2PdcError> {
        let transform_list: Result<Vec<TransformListToken>, svgtypes::Error> =
            svgtypes::TransformListParser::from(child.attribute("transform").unwrap_or(""))
                .collect();
        let transform_list = transform_list?;
        let translate = transform_list
            .into_iter()
            .find(|token| matches!(token, TransformListToken::Translate { .. }))
            .unwrap_or(TransformListToken::Translate { tx: 0.0, ty: 0.0 });
        let translate = match translate {
            TransformListToken::Translate { tx, ty } => FPoint::new(tx as f32, ty as f32),
            _ => FPoint::default(),
        };
        Ok(translate)
    }

    pub fn parse_svg_image(
        &self,
        content: &str,
        truncate_color: &TruncateColor,
        conversion: &Conversion,
    ) -> Svg2PdcResult<PebbleImage> {
        let root = roxmltree::Document::parse(content)?;
        let view_box = Self::get_viewbox(&root)?;
        let translation = FPoint {
            x: -view_box.x as f32,
            y: -view_box.y as f32,
        };
        let size = FPoint {
            x: view_box.w as f32,
            y: view_box.h as f32,
        }
        .pebble_coordinates(&self.precision, conversion)?;

        let commands = self.get_commands(
            &translation,
            truncate_color,
            &GroupOptions::default(),
            conversion,
            root.root_element(),
        )?;
        Ok(PebbleImage { size, commands })
    }
}
