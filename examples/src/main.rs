use morphing::mobjects::shape::Rect;
// use morphing::toplevel::config::Config;
use morphing::toplevel::scene::Scene;
use morphing::toplevel::scene::Supervisor;

pub struct Main;

impl Scene for Main {
    fn construct(self, sv: &Supervisor) {
        sv.wait(1.0);
        let mobject = sv.spawn(Rect {
            min: nalgebra::Vector2::new(-1.0, -1.0),
            max: nalgebra::Vector2::new(1.0, 1.0),
        });
        sv.wait(6.0);
        mobject.destroy();
        sv.wait(12.0);
    }
}

// fn main() -> anyhow::Result<()> {
//     Main.run(Config::default())
// }
