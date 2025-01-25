use morphing::mobjects::shape::Rect;
use morphing::timelines::alive::traits::Destroy;
use morphing::timelines::alive::Supervisor;
use morphing::toplevel::app::scene::Scene;
use morphing::toplevel::app::scene::Scenes;

struct DemoScene;

impl Scene for DemoScene {
    fn construct(self, sv: &Supervisor<'_>) {
        sv.wait(1.0);
        let mobject = sv.spawn(Rect(nalgebra::Vector2::new(1.0, 1.0)));
        sv.wait(6.0);
        mobject.destroy();
        sv.wait(12.0);
    }
}

fn main() {
    DemoScene.run();
}
