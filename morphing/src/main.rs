// use morphing::toplevel::app::App;

fn main() {
    evcxr::runtime_hook();
    let (mut ctx, outputs) = evcxr::CommandContext::new().unwrap();
    ctx.execute(r#":dep morphing = {path="E:/ManimKindergarten/Rust/morphing/morphing"}"#)
        .unwrap();
    ctx.execute(r#":dep examples = {path="E:/ManimKindergarten/Rust/morphing/examples"}"#)
        .unwrap();
    ctx.execute(r#"println!("{}", morphing::toplevel::app::ron::ser::to_string(&examples::demo_scene(morphing::toplevel::settings::SceneSettings::default())).unwrap());"#).unwrap();
    if let Ok(line) = outputs.stdout.recv() {
        println!("{line}");
    }
    // env_logger::init();
    // let event_loop = winit::event_loop::EventLoop::new().unwrap();
    // // event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    // event_loop.run_app(&mut App::new())
}
