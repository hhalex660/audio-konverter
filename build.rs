// Windows: Programm-Icon in die .exe einbauen (andere Systeme brauchen es nicht)
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").is_ok_and(|os| os == "windows") {
        let _ = winres::WindowsResource::new()
            .set_icon("assets/icon.ico")
            .compile();
    }
}
