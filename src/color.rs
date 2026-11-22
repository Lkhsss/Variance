//! 极简的 ANSI 着色实现。
//!
//! 输出前会检测终端与 `NO_COLOR`，非终端或禁用颜色时退化为纯文本。

use std::env;
use std::fmt::{self, Display};
use std::io::IsTerminal;
use std::sync::OnceLock;

/// 是否输出 ANSI 颜色序列（进程内只检测一次）
///
/// 以下条件全部满足才着色：
/// 1. 标准输出是终端（重定向到文件或管道时不着色）；
/// 2. 环境变量 `NO_COLOR` 不存在，或存在但为空（<https://no-color.org>）；
/// 3. `TERM` 不等于 `dumb`。
pub fn colors_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        should_colorize(
            std::io::stdout().is_terminal(),
            no_color_env_set(),
            term_is_dumb(),
        )
    })
}

/// 判定逻辑本身，抽成纯函数便于单元测试
fn should_colorize(is_terminal: bool, no_color: bool, term_is_dumb: bool) -> bool {
    is_terminal && !no_color && !term_is_dumb
}

/// `NO_COLOR` 存在且非空时返回 true（空值按规范视为未设置）
fn no_color_env_set() -> bool {
    env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty())
}

fn term_is_dumb() -> bool {
    env::var_os("TERM").is_some_and(|t| t.to_string_lossy() == "dumb")
}

/// 前景色：经典 SGR 色码，或 24 位真彩色
#[derive(Clone, Copy, Debug)]
pub enum Color {
    /// SGR 前景色码，如 31（红）、92（亮绿）
    Ansi(u8),
    /// 24 位真彩色
    Rgb(u8, u8, u8),
}

impl Color {
    pub const RED: Color = Color::Ansi(31);
    pub const YELLOW: Color = Color::Ansi(33);
    pub const BRIGHT_GREEN: Color = Color::Ansi(92);
}

/// 一段文本的显示样式
#[derive(Clone, Copy)]
pub struct Style {
    pub fg: Option<Color>,
    pub bold: bool,
}

impl Style {
    pub const RED: Style = Style {
        fg: Some(Color::RED),
        bold: false,
    };
    pub const YELLOW: Style = Style {
        fg: Some(Color::YELLOW),
        bold: false,
    };
    pub const BRIGHT_GREEN: Style = Style {
        fg: Some(Color::BRIGHT_GREEN),
        bold: false,
    };

    /// 真彩色（R, G, B）
    pub const fn truecolor(r: u8, g: u8, b: u8) -> Style {
        Style {
            fg: Some(Color::Rgb(r, g, b)),
            bold: false,
        }
    }

    /// 加粗（可与其它的样式链式组合）
    pub const fn bold(self) -> Style {
        Style {
            fg: self.fg,
            bold: true,
        }
    }
}

/// 包装器：携带被着色值和样式，负责输出 ANSI 序列
pub struct Colored<'a, T: ?Sized> {
    value: &'a T,
    style: Style,
}

impl<'a, T: Display + ?Sized> Colored<'a, T> {
    /// 追加加粗，支持 `.red().bold()` 这类链式调用
    pub fn bold(self) -> Colored<'a, T> {
        Colored {
            value: self.value,
            style: self.style.bold(),
        }
    }
}

impl<T: Display + ?Sized> Display for Colored<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_styled(f, self.value, self.style, colors_enabled())
    }
}

/// 按样式写出文本；`enabled` 为 false 时只写纯文本。
///
/// 抽成独立函数，便于在不依赖终端状态的情况下测试渲染结果。
fn write_styled<W: fmt::Write, T: Display + ?Sized>(
    w: &mut W,
    value: &T,
    style: Style,
    enabled: bool,
) -> fmt::Result {
    if !enabled {
        return write!(w, "{value}");
    }
    if style.bold {
        w.write_str("\x1b[1m")?;
    }
    match style.fg {
        Some(Color::Ansi(code)) => write!(w, "\x1b[{code}m")?,
        Some(Color::Rgb(r, g, b)) => write!(w, "\x1b[38;2;{r};{g};{b}m")?,
        None => {}
    }
    write!(w, "{value}")?;
    w.write_str("\x1b[0m") // 重置
}

/// 为任意 `Display` 类型提供着色能力
pub trait Colorize: Display {
    fn colorize(&self, style: Style) -> Colored<'_, Self> {
        Colored { value: self, style }
    }

    fn red(&self) -> Colored<'_, Self> {
        self.colorize(Style::RED)
    }

    fn yellow(&self) -> Colored<'_, Self> {
        self.colorize(Style::YELLOW)
    }

    fn bright_green(&self) -> Colored<'_, Self> {
        self.colorize(Style::BRIGHT_GREEN)
    }

    fn truecolor(&self, r: u8, g: u8, b: u8) -> Colored<'_, Self> {
        self.colorize(Style::truecolor(r, g, b))
    }
}

impl<T: Display + ?Sized> Colorize for T {}

#[cfg(test)]
mod tests {
    use super::*;

    fn render<T: Display + ?Sized>(value: &T, style: Style, enabled: bool) -> String {
        let mut out = String::new();
        write_styled(&mut out, value, style, enabled).unwrap();
        out
    }

    #[test]
    fn requires_terminal() {
        assert!(should_colorize(true, false, false));
        assert!(!should_colorize(false, false, false));
    }

    #[test]
    fn no_color_wins() {
        // NO_COLOR 存在且非空时，即使在终端里也不着色
        assert!(!should_colorize(true, true, false));
    }

    #[test]
    fn dumb_terminal_has_no_color() {
        assert!(!should_colorize(true, false, true));
    }

    #[test]
    fn renders_ansi_color() {
        assert_eq!(render(&"x", Style::RED, true), "\x1b[31mx\x1b[0m");
        assert_eq!(render(&"x", Style::BRIGHT_GREEN, true), "\x1b[92mx\x1b[0m");
    }

    #[test]
    fn renders_truecolor() {
        assert_eq!(
            render(&"x", Style::truecolor(0, 191, 255), true),
            "\x1b[38;2;0;191;255mx\x1b[0m"
        );
    }

    #[test]
    fn renders_bold_then_color() {
        assert_eq!(
            render(&"x", Style::RED.bold(), true),
            "\x1b[1m\x1b[31mx\x1b[0m"
        );
    }

    #[test]
    fn disabled_renders_plain_text() {
        assert_eq!(render(&"x", Style::RED.bold(), false), "x");
        assert_eq!(render(&"x", Style::truecolor(1, 2, 3), false), "x");
    }
}
