use std::sync::atomic::{AtomicUsize, Ordering};

use chrono::Local;
use colored::Colorize;
use fern::colors::{Color, ColoredLevelConfig};
use fern::Dispatch;
use log::LevelFilter;

static MAX_MODULE_WIDTH: AtomicUsize = AtomicUsize::new(0);

pub const DEFAULT_LEVEL: LevelFilter = LevelFilter::Info;

/// Retrieve and set max module width.
fn max_module_width(target: &str) -> usize {
    let mut max_width = MAX_MODULE_WIDTH.load(Ordering::Acquire);
    if max_width < target.len() {
        MAX_MODULE_WIDTH.store(target.len(), Ordering::Release);
        max_width = target.len();
    }
    max_width
}

/// Trait that implements new behavior for fern's Dispatch.
pub trait AlbatrossDispatch {
    /// Setup logging in pretty_env_logger style.
    fn pretty_logging(self, show_timestamps: bool) -> Self;
}

fn pretty_logging(dispatch: Dispatch, colors_level: ColoredLevelConfig) -> Dispatch {
    dispatch.format(move |out, message, record| {
        let max_width = max_module_width(record.target());
        let target = format!("{: <width$}", record.target(), width=max_width);
        out.finish(format_args!(
            " {level} {target} > {message}",
            target = target.bold(),
            level = colors_level.color(record.level()),
            message = message,
        ));
    })
}

fn pretty_logging_with_timestamps(dispatch: Dispatch, colors_level: ColoredLevelConfig) -> Dispatch {
    dispatch.format(move |out, message, record| {
        let max_width = max_module_width(record.target());
        let target = format!("{: <width$}", record.target(), width=max_width);
        out.finish(format_args!(
            " {timestamp} {level} {target} > {message}",
            timestamp = Local::now().format("%Y-%m-%d %H:%M:%S"),
            target = target.bold(),
            level = colors_level.color(record.level()),
            message = message,
        ));
    })
}

impl AlbatrossDispatch for Dispatch {
    fn pretty_logging(self, show_timestamps: bool) -> Self {
        let colors_level = ColoredLevelConfig::new()
            .error(Color::Red)
            .warn(Color::Yellow)
            .info(Color::Green)
            .debug(Color::Blue)
            .trace(Color::Magenta);

        if show_timestamps {
            pretty_logging_with_timestamps(self, colors_level)
        } else {
            pretty_logging(self, colors_level)
        }
    }
}
