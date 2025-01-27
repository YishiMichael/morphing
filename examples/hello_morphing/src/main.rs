use morphing::mobjects::shape::Rect;
use morphing::timelines::alive::traits::Destroy;
use morphing::timelines::alive::Supervisor;
use morphing::toplevel::scene::scene;
use morphing::toplevel::settings::SceneSettings;

fn override_settings(scene_settings: SceneSettings) -> SceneSettings {
    scene_settings
}

#[scene]
pub fn demo_scene(sv: &Supervisor<'_>) {
    sv.wait(1.0);
    let mobject = sv.spawn(Rect(nalgebra::Vector2::new(1.0, 1.0)));
    sv.wait(6.0);
    mobject.destroy();
    sv.wait(12.0);
}

#[scene(override_settings = "override_settings")]
pub fn another_demo_scene(sv: &Supervisor<'_>) {
    sv.wait(1.0);
    let mobject = sv.spawn(Rect(nalgebra::Vector2::new(1.0, 1.0)));
    sv.wait(6.0);
    mobject.destroy();
    sv.wait(10.0);
}

fn main() {
    morphing::toplevel::scene::export_scenes();
}
