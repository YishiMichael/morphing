use morphing::mobjects::shape::Rect;
use morphing::timelines::alive::traits::Destroy;
use morphing::timelines::timeline::Supervisor;
use morphing::toplevel::scene::export_scenes;
use morphing::toplevel::scene::scene;
use morphing::toplevel::scene::SceneFilter;

#[scene]
fn demo_scene(sv: &Supervisor<'_>) {
    sv.wait(1.0);
    let mobject = sv.spawn(Rect(nalgebra::Vector2::new(1.0, 1.0)));
    sv.wait(6.0);
    mobject.destroy();
    sv.wait(12.0);
}

#[scene(config_path = "my_config.toml")]
fn another_demo_scene(sv: &Supervisor<'_>) {
    sv.wait(1.0);
    let mobject = sv.spawn(Rect(nalgebra::Vector2::new(1.0, 1.0)));
    sv.wait(6.0);
    mobject.destroy();
    sv.wait(10.0);
}

fn main() {
    export_scenes(SceneFilter::All);
}
