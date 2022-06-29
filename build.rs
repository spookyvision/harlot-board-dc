use std::{process::Command, env};

// Necessary because of this issue: https://github.com/rust-lang/cargo/issues/9641
fn main() -> anyhow::Result<()> {
    // if let Err(e) = Command::new("zopfli").arg(p.path()).output() {
    //                 eprintln!("!! {e:?}");
    //             }

    let out_dir = env::var("OUT_DIR").unwrap();
    let mut cmd = Command::new("../../GitHub/color-mixer-ws/target/debug/pack.exe");
    let cmd = cmd.args([
         &out_dir
         , "../../GitHub/color-mixer-ws/mixer-dioxus/dist/"
    ]);
    cmd.output().unwrap();

    embuild::build::CfgArgs::output_propagated("ESP_IDF")?;
    embuild::build::LinkArgs::output_propagated("ESP_IDF")
}
