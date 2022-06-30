use std::{env, process::Command};

// Necessary because of this issue: https://github.com/rust-lang/cargo/issues/9641
fn main() -> anyhow::Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_file = out_dir + "/../../web_includes.rs";
    // WSL paths are too bork, giving up
    let out_file = "web_includes.rs";

    // cursed paths are cursed
    let mut cmd = Command::new("../color-mixer-ws/target/debug/pack.exe");

    let cmd = cmd.args([&out_file, "../color-mixer-ws/mixer-dioxus/dist/"]);

    let argstr: Vec<String> = cmd
        .get_args()
        .map(|arg| arg.to_string_lossy().to_string())
        .collect();
    let argstr = argstr.join("\n");
    let cmd_and_args = cmd.get_program().to_string_lossy().to_string() + &argstr;

    let output = cmd.output().unwrap();

    embuild::build::CfgArgs::output_propagated("ESP_IDF")?;
    embuild::build::LinkArgs::output_propagated("ESP_IDF")
}
