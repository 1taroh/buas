use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, Instant};
use uuid::Uuid;

const DEFAULT_FILE_COUNT: u64 = 2_048;
const DEFAULT_FILE_SIZE: u64 = 256 * 1024;
const DEFAULT_DIRECTORY_COUNT: u64 = 64;

struct TestPaths {
    root: PathBuf,
    dram_generation: Option<PathBuf>,
}

impl TestPaths {
    fn new() -> io::Result<Self> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target/buas-many-files-tests")
            .join(Uuid::new_v4().to_string());
        fs::create_dir_all(&root)?;

        Ok(Self {
            root,
            dram_generation: None,
        })
    }

    fn cleanup(&mut self) -> io::Result<()> {
        if let Some(generation) = self.dram_generation.take()
            && generation.exists()
        {
            fs::remove_dir_all(generation)?;
        }
        if self.root.exists() {
            fs::remove_dir_all(&self.root)?;
        }
        Ok(())
    }
}

impl Drop for TestPaths {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .map(|value| value.parse::<u64>())
        .transpose()
        .unwrap_or_else(|_| panic!("{name} must be a non-negative integer"))
        .unwrap_or(default)
}

fn run_and_measure(command: &mut Command) -> io::Result<(Output, Duration)> {
    let started = Instant::now();
    let output = command.output()?;
    Ok((output, started.elapsed()))
}

fn assert_command_succeeded(label: &str, output: &Output) {
    assert!(
        output.status.success(),
        "{label} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

fn assert_file_is_zero_filled(path: &Path, expected_size: u64) -> io::Result<()> {
    assert_eq!(fs::metadata(path)?.len(), expected_size, "{path:?}");

    let mut file = File::open(path)?;
    let mut buffer = vec![0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        assert!(
            buffer[..read].iter().all(|byte| *byte == 0),
            "non-zero data in {path:?}"
        );
    }
    Ok(())
}

fn assert_generated_tree(
    root: &Path,
    file_count: u64,
    file_size: u64,
    directory_count: u64,
) -> io::Result<()> {
    for index in 0..file_count {
        let path = root
            .join("lib")
            .join(format!("package-{}", index % directory_count))
            .join(format!("file-{index}.bin"));
        assert_file_is_zero_filled(&path, file_size)?;
    }

    let actual_file_count =
        fs::read_dir(root.join("lib"))?.try_fold(0_u64, |count, directory| -> io::Result<u64> {
            let directory = directory?;
            Ok(count + fs::read_dir(directory.path())?.count() as u64)
        })?;
    assert_eq!(actual_file_count, file_count);
    Ok(())
}

#[test]
#[ignore = "writes many files to both project storage and /dev/shm"]
fn compares_many_file_writes_to_ssd_and_dram_and_cleans_up() -> io::Result<()> {
    let file_count = env_u64("BUAS_MANY_FILES_COUNT", DEFAULT_FILE_COUNT);
    let file_size = env_u64("BUAS_MANY_FILES_SIZE", DEFAULT_FILE_SIZE);
    let directory_count = env_u64("BUAS_MANY_FILES_DIRECTORIES", DEFAULT_DIRECTORY_COUNT);
    assert!(
        directory_count > 0,
        "directory count must be greater than zero"
    );

    let mut paths = TestPaths::new()?;
    let ssd_dir = paths.root.join("ssd");
    let dram_project_dir = paths.root.join("dram-project");
    fs::create_dir_all(&ssd_dir)?;
    fs::create_dir_all(&dram_project_dir)?;

    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/generate_dummy_files.sh");
    let arguments = [
        "dataset".to_owned(),
        file_count.to_string(),
        file_size.to_string(),
        directory_count.to_string(),
    ];

    let (ssd_output, ssd_elapsed) = run_and_measure(
        Command::new(&fixture)
            .args(&arguments)
            .current_dir(&ssd_dir),
    )?;
    assert_command_succeeded("project-storage write", &ssd_output);

    let (buas_output, dram_elapsed) = run_and_measure(
        Command::new(env!("CARGO_BIN_EXE_buas"))
            .arg(&fixture)
            .args(&arguments)
            .current_dir(&dram_project_dir),
    )?;
    let generation_id = String::from_utf8_lossy(&buas_output.stdout)
        .lines()
        .next()
        .expect("buas did not print a generation UUID")
        .to_owned();
    paths.dram_generation = Some(Path::new("/dev/shm/buas").join(generation_id));
    assert_command_succeeded("DRAM write through buas", &buas_output);

    let ssd_dataset = ssd_dir.join("dataset");
    let exposed_dataset = dram_project_dir.join("dataset");
    assert!(!fs::symlink_metadata(&ssd_dataset)?.file_type().is_symlink());
    assert!(
        fs::symlink_metadata(&exposed_dataset)?
            .file_type()
            .is_symlink()
    );

    let generation = paths
        .dram_generation
        .as_ref()
        .expect("generation path must have been recorded");
    assert_eq!(fs::read_link(&exposed_dataset)?, generation.join("dataset"));
    assert_generated_tree(&ssd_dataset, file_count, file_size, directory_count)?;
    assert_generated_tree(&exposed_dataset, file_count, file_size, directory_count)?;

    let total_mib = file_count.saturating_mul(file_size) as f64 / (1024.0 * 1024.0);
    println!(
        "many-files comparison ({file_count} files, {file_size} bytes each, {total_mib:.1} MiB per destination)"
    );
    println!("  project storage: {ssd_elapsed:.3?}");
    println!("  buas on DRAM:    {dram_elapsed:.3?}");

    let generation = generation.clone();
    let root = paths.root.clone();
    paths.cleanup()?;
    assert!(!generation.exists(), "DRAM generation was not removed");
    assert!(!root.exists(), "project-storage test data was not removed");

    Ok(())
}
