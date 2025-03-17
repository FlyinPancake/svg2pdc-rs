use anyhow::Ok;
use svg2pdc::{point::Conversion, prelude::*};

const SVG_SOURCES: [&str; 9] = [
    "tests/resources/svg/Pebble_50x50_Generic_weather.svg",
    "tests/resources/svg/Pebble_50x50_Heavy_rain.svg",
    "tests/resources/svg/Pebble_50x50_Heavy_snow.svg",
    "tests/resources/svg/Pebble_50x50_Light_rain.svg",
    "tests/resources/svg/Pebble_50x50_Light_snow.svg",
    "tests/resources/svg/Pebble_50x50_Partly_cloudy.svg",
    "tests/resources/svg/Pebble_50x50_Sunny_day.svg",
    "tests/resources/svg/pencil-illustrator.svg",
    "tests/resources/svg/pencil-inkscape.svg",
];

fn test_svg_conversion(svg: &str) -> anyhow::Result<()> {
    // eprintln!("Testing: {}", svg);
    let svg_path = std::path::Path::new(svg);
    let pdc_path =
        std::path::Path::new("tests/resources/golden_pdc/").join(svg_path.file_stem().unwrap());
    let pdc_path = pdc_path.with_extension("pdc");

    let svg_content = std::fs::read_to_string(svg_path)?;
    // eprintln!("SVG content: {}", svg_content);
    let converter = SvgConverter::new(Precision::Normal);

    let image = converter.parse_svg_image(
        &svg_content,
        &TruncateColor::Truncate,
        &Conversion::RequireExact,
    )?;
    let mut converted_pdc = Vec::new();
    image.serialize(&mut converted_pdc)?;

    let original_pdc_content = std::fs::read(pdc_path)?;

    assert_eq!(converted_pdc, original_pdc_content);
    Ok(())
}

#[test]
fn test_generic_weather() -> anyhow::Result<()> {
    test_svg_conversion(SVG_SOURCES[0])
}

#[test]
fn test_heavy_rain() -> anyhow::Result<()> {
    test_svg_conversion(SVG_SOURCES[1])
}

#[test]
fn test_heavy_snow() -> anyhow::Result<()> {
    test_svg_conversion(SVG_SOURCES[2])
}

#[test]
fn test_light_rain() -> anyhow::Result<()> {
    test_svg_conversion(SVG_SOURCES[3])
}

#[test]
fn test_light_snow() -> anyhow::Result<()> {
    test_svg_conversion(SVG_SOURCES[4])
}

#[test]
fn test_partly_cloudy() -> anyhow::Result<()> {
    test_svg_conversion(SVG_SOURCES[5])
}

#[test]
fn test_sunny_day() -> anyhow::Result<()> {
    test_svg_conversion(SVG_SOURCES[6])
}

#[test]
fn test_pencil_illustrator() -> anyhow::Result<()> {
    let svg = SVG_SOURCES[7];
    let svg_path = std::path::Path::new(svg);
    let pdc_path =
        std::path::Path::new("tests/resources/golden_pdc/").join(svg_path.file_stem().unwrap());
    let pdc_path = pdc_path.with_extension("pdc");

    let svg_content = std::fs::read_to_string(svg_path)?;
    let converter = SvgConverter::new(Precision::Normal);

    let image = converter.parse_svg_image(
        &svg_content,
        &TruncateColor::Truncate,
        &Conversion::ConvertNoWarn,
    )?;
    let mut converted_pdc = Vec::new();
    image.serialize(&mut converted_pdc)?;

    let original_pdc_content = std::fs::read(pdc_path)?;

    assert_eq!(converted_pdc, original_pdc_content);
    Ok(())
}

#[test]
fn test_pencil_inkscape() -> anyhow::Result<()> {
    let svg = SVG_SOURCES[8];
    let svg_path = std::path::Path::new(svg);
    let pdc_path =
        std::path::Path::new("tests/resources/golden_pdc/").join(svg_path.file_stem().unwrap());
    let pdc_path = pdc_path.with_extension("pdc");

    let svg_content = std::fs::read_to_string(svg_path)?;
    let converter = SvgConverter::new(Precision::Normal);

    let image = converter.parse_svg_image(
        &svg_content,
        &TruncateColor::Truncate,
        &Conversion::ConvertNoWarn,
    )?;
    let mut converted_pdc = Vec::new();
    image.serialize(&mut converted_pdc)?;

    let original_pdc_content = std::fs::read(pdc_path)?;

    assert_eq!(converted_pdc, original_pdc_content);
    Ok(())
}
