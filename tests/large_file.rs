use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, Instant};
use uuid::Uuid;

const DEFAULT_SIZE_MIB: u64 = 512;

struct TestPaths {
    root: PathBuf,
    dram_generation: Option<PathBuf>,
}

impl TestPaths {
    fn new() -> io::Result<Self> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target/buas-large-file-tests")
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
    assert_eq!(fs::metadata(path)?.len(), expected_size);

    let mut file = File::open(path)?;
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        assert!(buffer[..read].iter().all(|byte| *byte == 0));
    }
    Ok(())
}

#[test]
#[ignore = "writes a large file to both project storage and /dev/shm"]
fn compares_large_file_write_to_ssd_and_dram_and_cleans_up() -> io::Result<()> {
    let size_mib = std::env::var("BUAS_LARGE_TEST_MIB")
        .ok()
        .map(|value| value.parse::<u64>())
        .transpose()
        .expect("BUAS_LARGE_TEST_MIB must be a non-negative integer")
        .unwrap_or(DEFAULT_SIZE_MIB);
    let expected_size = size_mib * 1024 * 1024;
    let mut paths = TestPaths::new()?;

    let ssd_dir = paths.root.join("ssd");
    let dram_project_dir = paths.root.join("dram-project");
    fs::create_dir_all(&ssd_dir)?;
    fs::create_dir_all(&dram_project_dir)?;

    let (ssd_output, ssd_elapsed) = run_and_measure(
        Command::new("dd")
            .args([
                "if=/dev/zero",
                "of=large.bin",
                "bs=1M",
                &format!("count={size_mib}"),
                "conv=fsync",
                "status=none",
            ])
            .current_dir(&ssd_dir),
    )?;
    assert_command_succeeded("SSD write", &ssd_output);

    let (buas_output, dram_elapsed) = run_and_measure(
        Command::new(env!("CARGO_BIN_EXE_buas"))
            .args([
                "dd",
                "if=/dev/zero",
                "of=large.bin",
                "bs=1M",
                &format!("count={size_mib}"),
                "conv=fsync",
                "status=none",
            ])
            .current_dir(&dram_project_dir),
    )?;

    let generation_id = String::from_utf8_lossy(&buas_output.stdout)
        .lines()
        .next()
        .expect("buas did not print a generation UUID")
        .to_owned();
    paths.dram_generation = Some(Path::new("/dev/shm/buas").join(generation_id));
    assert_command_succeeded("DRAM write through buas", &buas_output);

    let ssd_file = ssd_dir.join("large.bin");
    let exposed_file = dram_project_dir.join("large.bin");
    assert!(!fs::symlink_metadata(&ssd_file)?.file_type().is_symlink());
    assert!(
        fs::symlink_metadata(&exposed_file)?
            .file_type()
            .is_symlink()
    );

    let link_target = fs::read_link(&exposed_file)?;
    let generation = paths
        .dram_generation
        .as_ref()
        .expect("generation path must have been recorded");
    assert_eq!(link_target, generation.join("large.bin"));
    assert_file_is_zero_filled(&ssd_file, expected_size)?;
    assert_file_is_zero_filled(&exposed_file, expected_size)?;

    println!("large-file comparison ({size_mib} MiB per destination)");
    println!("  project storage: {ssd_elapsed:.3?}");
    println!("  buas on DRAM:    {dram_elapsed:.3?}");

    let generation = generation.clone();
    let root = paths.root.clone();
    paths.cleanup()?;
    assert!(!generation.exists(), "DRAM generation was not removed");
    assert!(!root.exists(), "project-storage test data was not removed");

    Ok(())
}
