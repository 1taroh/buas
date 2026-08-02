use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;

fn expose_dram_workspace_entries(dram_workspace: &Path, project_dir: &Path) -> std::io::Result<()> {
    // UUID直下のエントリを current dir に symlink として公開
    for entry in fs::read_dir(dram_workspace)? {
        let entry = entry?;
        symlink(entry.path(), project_dir.join(entry.file_name()))?;
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    let mut args = std::env::args().skip(1);
    let current_dir = std::env::current_dir()?;

    let id = Uuid::new_v4();
    println!("{id}");

    let dir = PathBuf::from(format!("/dev/shm/buas/{id}"));
    fs::create_dir_all(&dir)?;

    let program = args.next().expect("コマンドが指定されていません");
    let program_path = Path::new(&program);
    let program = if program_path.is_absolute() || !program.contains('/') {
        program.into()
    } else {
        std::env::current_dir()?.join(program)
    };

    let status = Command::new(program)
        .args(args)
        .current_dir(&dir)
        .status()?;

    println!("{status}");

    // 作業領域 (dram) 直下の成果物を、元のカレントディレクトリから参照できるようにする。
    expose_dram_workspace_entries(&dir, &current_dir)?;

    Ok(())
}
