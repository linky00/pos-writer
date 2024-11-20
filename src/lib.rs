use codepage_437::{ToCp437, CP437_CONTROL};
use escpos::{
    driver::Driver,
    errors::PrinterError,
    printer::Printer,
    utils::{Font, JustifyMode, UnderlineMode},
};

pub struct Style {
    pub layers: Vec<StyleLayer>,
}

impl Style {
    pub fn new(layers: Vec<StyleLayer>) -> Self {
        Style { layers }
    }
}

pub enum StyleLayer {
    Font(Font),
    Size((u8, u8)),
    Bold,
    Underline(UnderlineMode),
    Justify(JustifyMode),
    UpsideDown,
    Reverse,
    DoubleStrike,
    LineSpacing(u8),
}

pub struct TextBox {
    wrap_chars: Option<u32>,
    border_type: Option<BorderType>,
}

impl TextBox {
    pub fn new(wrap_chars: Option<u32>, border_type: Option<BorderType>) -> Self {
        TextBox {
            wrap_chars,
            border_type,
        }
    }
}

pub enum BorderType {
    Single,
    Double,
    LightShade,
    MediumShade,
    DarkShade,
    Black,
}

struct BorderCharacters {
    top_left: char,
    top: char,
    top_right: char,
    left: char,
    right: char,
    bottom_left: char,
    bottom: char,
    bottom_right: char,
}

const SINGLE_BORDER_CHARS: BorderCharacters = BorderCharacters {
    top_left: '┌',
    top: '─',
    top_right: '┐',
    left: '│',
    right: '│',
    bottom_left: '└',
    bottom: '─',
    bottom_right: '┘',
};

const DOUBLE_BORDER_CHARS: BorderCharacters = BorderCharacters {
    top_left: '╔',
    top: '═',
    top_right: '╗',
    left: '║',
    right: '║',
    bottom_left: '╚',
    bottom: '═',
    bottom_right: '╝',
};

const LIGHT_SHADE_BORDER_CHARS: BorderCharacters = BorderCharacters {
    top_left: '░',
    top: '░',
    top_right: '░',
    left: '░',
    right: '░',
    bottom_left: '░',
    bottom: '░',
    bottom_right: '░',
};

const MEDIUM_SHADE_BORDER_CHARS: BorderCharacters = BorderCharacters {
    top_left: '▒',
    top: '▒',
    top_right: '▒',
    left: '▒',
    right: '▒',
    bottom_left: '▒',
    bottom: '▒',
    bottom_right: '▒',
};

const DARK_SHADE_BORDER_CHARS: BorderCharacters = BorderCharacters {
    top_left: '▓',
    top: '▓',
    top_right: '▓',
    left: '▓',
    right: '▓',
    bottom_left: '▓',
    bottom: '▓',
    bottom_right: '▓',
};

const BLACK_BORDER_CHARS: BorderCharacters = BorderCharacters {
    top_left: '▄',
    top: '▄',
    top_right: '▄',
    left: '▐',
    right: '▌',
    bottom_left: '▀',
    bottom: '▀',
    bottom_right: '▀',
};

pub fn print_line_with_style<D: Driver>(
    printer: &mut Printer<D>,
    style: &Style,
    text: &str,
) -> Result<(), PrinterError> {
    set_style(printer, style)?;
    print_line(printer, text)?;
    undo_style(printer, style)?;

    Ok(())
}

pub fn print_line_with_style_box<D: Driver>(
    printer: &mut Printer<D>,
    style: &Style,
    text: &str,
    text_box: &TextBox,
) -> Result<(), PrinterError> {
    set_style(printer, style)?;

    let mut lines = vec![];

    if let Some(max_width) = text_box.wrap_chars {
        let mut current_line = String::from("");
        for word in text.split(' ') {
            if current_line.len() + word.len() + 1 > max_width as usize {
                if current_line.len() == 0 {
                    lines.push(String::from(word));
                } else {
                    lines.push(current_line);
                    current_line = String::from(word);
                }
            } else {
                if current_line.len() != 0 {
                    current_line += " ";
                }
                current_line += word;
            }
        }
        if !current_line.is_empty() {
            lines.push(current_line);
        }
    } else {       
        lines.push(text.to_string());
    }

    if let Some(border_type) = &text_box.border_type {
        let border_characters = match border_type {
            BorderType::Single => SINGLE_BORDER_CHARS,
            BorderType::Double => DOUBLE_BORDER_CHARS,
            BorderType::LightShade => LIGHT_SHADE_BORDER_CHARS,
            BorderType::MediumShade => MEDIUM_SHADE_BORDER_CHARS,
            BorderType::DarkShade => DARK_SHADE_BORDER_CHARS,
            BorderType::Black => BLACK_BORDER_CHARS,
        };

        let max_width = lines
            .iter()
            .map(|line| line.len())
            .max()
            .unwrap_or_default();

        lines = lines
            .iter()
            .map(|line| {
                format!(
                    "{} {}{} {}",
                    border_characters.left,
                    line,
                    " ".repeat(max_width - line.len()),
                    border_characters.right
                )
            })
            .collect();

        let vertical_line = |left, middle, right| {
            format!(
                "{}{}{}",
                left,
                String::from(middle).repeat(max_width + 2),
                right,
            )
        };

        let box_top = vertical_line(
            border_characters.top_left,
            border_characters.top,
            border_characters.top_right,
        );
        let box_bottom = vertical_line(
            border_characters.bottom_left,
            border_characters.bottom,
            border_characters.bottom_right,
        );
        lines.insert(0, box_top);
        lines.push(box_bottom);
    }

    for line in lines {
        print_line(printer, &line)?;
    }

    undo_style(printer, style)?;

    Ok(())
}

pub fn print_line<D: Driver>(printer: &mut Printer<D>, line: &str) -> Result<(), PrinterError> {
    print(printer, line)?;
    printer.feed()?;

    Ok(())
}

pub fn print<D: Driver>(printer: &mut Printer<D>, line: &str) -> Result<(), PrinterError> {
    let in_cp437 = line
        .to_cp437(&CP437_CONTROL)
        .expect("All characters should be valid Codepage 437");
    printer.custom(&in_cp437)?;

    Ok(())
}

pub fn set_style<D: Driver>(printer: &mut Printer<D>, style: &Style) -> Result<(), PrinterError> {
    for layer in &style.layers {
        match layer {
            StyleLayer::Font(font) => match font {
                &Font::A => printer.font(Font::A),
                &Font::B => printer.font(Font::B),
                &Font::C => printer.font(Font::C),
            }?,
            StyleLayer::Size((width, height)) => printer.size(*width, *height)?,
            StyleLayer::Bold => printer.bold(true)?,
            StyleLayer::Underline(mode) => match mode {
                &UnderlineMode::None => printer.underline(UnderlineMode::None),
                &UnderlineMode::Single => printer.underline(UnderlineMode::Single),
                &UnderlineMode::Double => printer.underline(UnderlineMode::Double),
            }?,
            StyleLayer::Justify(mode) => printer.justify(*mode)?,
            StyleLayer::UpsideDown => printer.upside_down(true)?,
            StyleLayer::Reverse => printer.reverse(true)?,
            StyleLayer::DoubleStrike => printer.double_strike(true)?,
            StyleLayer::LineSpacing(spacing) => printer.line_spacing(*spacing)?,
        };
    }

    Ok(())
}

pub fn undo_style<D: Driver>(printer: &mut Printer<D>, style: &Style) -> Result<(), PrinterError> {
    for layer in &style.layers {
        match layer {
            StyleLayer::Font(_) => printer.font(Font::A)?,
            StyleLayer::Size(_) => printer.reset_size()?,
            StyleLayer::Bold => printer.bold(false)?,
            StyleLayer::Underline(_) => printer.underline(UnderlineMode::None)?,
            StyleLayer::Justify(_) => printer.justify(JustifyMode::LEFT)?,
            StyleLayer::UpsideDown => printer.upside_down(false)?,
            StyleLayer::Reverse => printer.reverse(false)?,
            StyleLayer::DoubleStrike => printer.double_strike(false)?,
            StyleLayer::LineSpacing(_) => printer.reset_line_spacing()?,
        };
    }

    Ok(())
}
