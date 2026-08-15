//! 2026-08-15 時点の Phase 1 初期プロトタイプ。
//!
//! 参照用の退避ファイルであり、現在のビルド対象ではない。

use std::fmt;
use std::fs;
use std::io;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;

#[derive(Debug)]
enum BuasError {
    MissingCommand,

    CreateWorkspace {
        path: PathBuf,
        source: io::Error,
    },

    SpawnCommand {
        program: PathBuf,
        source: io::Error,
    },

    ReadWorkspace {
        path: PathBuf,
        source: io::Error,
    },
    RemoveWorkspace {
        path: PathBuf,
        source: io::Error,
    },
    RemoveSymlink {
        path: PathBuf,
        source: io::Error,
    },
    PublishEntry {
        source_dir: PathBuf,
        destination_dir: PathBuf,
        error: io::Error,
    },
    CurrentDirectory {
        source: io::Error,
    },
}

fn expose_dram_workspace_entries(
    dram_workspace: &Path,
    project_dir: &Path,
) -> Result<Vec<PathBuf>, (BuasError, Vec<PathBuf>)> {
    let entries = fs::read_dir(dram_workspace).map_err(|source| {
        (
            BuasError::ReadWorkspace {
                path: dram_workspace.to_path_buf(),
                source,
            },
            Vec::new(),
        )
    })?;

    let mut published = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(source) => {
                return Err((
                    BuasError::ReadWorkspace {
                        path: dram_workspace.to_path_buf(),
                        source,
                    },
                    published,
                ));
            }
        };

        let source = entry.path();
        let destination = project_dir.join(entry.file_name());

        if let Err(error) = symlink(&source, &destination) {
            return Err((
                BuasError::PublishEntry {
                    source_dir: source,
                    destination_dir: destination,
                    error,
                },
                published,
            ));
        }

        published.push(destination);
    }

    Ok(published)
}

fn cleanup_dram_workspace(dram_workspace: &Path) -> Result<(), BuasError> {
    fs::remove_dir_all(dram_workspace).map_err(|source| BuasError::RemoveWorkspace {
        path: dram_workspace.to_path_buf(),
        source,
    })
}

// TODO
fn cleanup_project_symlinks(
    published: &Vec<PathBuf>
) -> Result<(), BuasError> {
    for symlink in published {
        fs::remove_file(symlink).map_err(|source| BuasError::RemoveSymlink {
            path: symlink.to_path_buf(),
            source,
        })?;
    }
    Ok(())
}

impl fmt::Display for BuasError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingCommand => {
                write!(formatter, "コマンドが指定されていません")
            }
            Self::CreateWorkspace { path, source } => {
                write!(
                    formatter,
                    "DRAM workspace {} を作成できません: {source}",
                    path.display()
                )
            }
            Self::SpawnCommand { program, source } => {
                write!(
                    formatter,
                    "コマンド {} を起動できません: {source}",
                    program.display()
                )
            }
            Self::ReadWorkspace { path, source } => {
                write!(
                    formatter,
                    "workspace {} を読み取れません: {source}",
                    path.display()
                )
            }
            Self::RemoveWorkspace { path, source } => {
                write!(
                    formatter,
                    "DRAM workspace {} を削除できません: {source}",
                    path.display()
                )
            }
            Self::RemoveSymlink {path, source} => {
                write!(
                    formatter,
                    "symlink {} を削除できません: {source}",
                    path.display()
                )
            }
            Self::PublishEntry {
                source_dir: source,
                destination_dir: destination,
                error,
            } => {
                write!(
                    formatter,
                    "{} を {} として公開できません: {error}",
                    source.display(),
                    destination.display()
                )
            }
            Self::CurrentDirectory { source } => {
                write!(
                    formatter,
                    "現在の作業ディレクトリを取得できません: {source}"
                )
            }
        }
    }
}

impl std::error::Error for BuasError {}

fn run() -> Result<std::process::ExitStatus, BuasError> {
    let mut args = std::env::args().skip(1);
    let program = args.next().ok_or(BuasError::MissingCommand)?;

    let current_dir =
        std::env::current_dir().map_err(|source| BuasError::CurrentDirectory { source })?;

    let id = Uuid::new_v4();
    println!("{id}");

    let dir = PathBuf::from(format!("/dev/shm/buas/{id}"));
    fs::create_dir_all(&dir).map_err(|source| BuasError::CreateWorkspace {
        path: dir.clone(),
        source,
    })?;

    let program_path = Path::new(&program);
    let program = if program_path.is_absolute() || !program.contains('/') {
        program.into()
    } else {
        std::env::current_dir()
            .map_err(|source| BuasError::CurrentDirectory { source })?
            .join(program)
    };

    let status = Command::new(&program)
        .args(args)
        .current_dir(&dir)
        .status()
        .map_err(|source| BuasError::SpawnCommand {
            program: program.clone(),
            source,
        })?;

    println!("{status}");

    // 作業領域 (dram) 直下の成果物を、元のカレントディレクトリから参照できるようにする。
    if !status.success() {
        cleanup_dram_workspace(&dir)?;
        return Ok(status);
    }
    match expose_dram_workspace_entries(&dir, &current_dir) {
        Ok(published) => published,
        Err((original_error, published)) => {
            cleanup_project_symlinks(&published)?;
            cleanup_dram_workspace(&dir)?;
            return Err(original_error);
        }
    };
    Ok(status)
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(status) => match status.code() {
            Some(code) => std::process::ExitCode::from(code as u8),
            None => std::process::ExitCode::FAILURE,
        },
        Err(error) => {
            eprintln!("buas: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
