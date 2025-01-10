pub trait Mobject: 'static + Clone {
    fn render(&self) {
        println!("Rendered!")
    }
}

impl Mobject for () {
    fn render(&self) {}
}
