use std::fs;
use std::process::Command;
use uuid::Uuid;

fn main() -> std::io::Result<()> {
    let mut args = std::env::args().skip(1);

    let id = Uuid::new_v4();
    println!("{id}");

    let dir = format!("/dev/shm/buas/{id}");
    fs::create_dir_all(&dir)?;

    let program = args.next().expect("コマンドが指定されていません");
    let status = Command::new(program)
        .args(args)
        .current_dir(&dir)
        .status()?;

    println!("{status}");
    Ok(())
}
