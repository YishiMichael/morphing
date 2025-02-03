#[derive(Debug, Default)]
pub(crate) struct Logger(Vec<(LoggingCategory, String)>);

#[derive(Clone, Copy, Debug)]
pub(crate) enum LoggingCategory {
    Stdin,
    Stdout,
    Stderr,
}

impl Logger {
    pub(crate) fn log_line(&mut self, category: LoggingCategory, line: String) {
        self.0.push((category, line));
    }

    pub(crate) fn log_lines<I>(&mut self, category: LoggingCategory, lines: I)
    where
        I: IntoIterator<Item = String>,
    {
        self.0
            .extend(lines.into_iter().map(|line| (category, line)));
    }
}
