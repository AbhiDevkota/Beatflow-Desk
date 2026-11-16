// Compiles the .slint UI tree into Rust (app-window.slint pulls in the rest
// via relative imports; fonts and icons are embedded at compile time).
fn main() {
    slint_build::compile("ui/app-window.slint").expect("failed to compile the Slint UI");
}
