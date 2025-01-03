use std::io::Write;

pub mod fill;
pub mod mobject;
pub mod path;
pub mod stroke;
pub mod typst;

// TODO: anyhow
pub fn render(a: String) -> std::io::Result<()> {
    let mut stdout = std::io::stdout();
    stdout.write(a.as_bytes()).map(|_| ())
}
