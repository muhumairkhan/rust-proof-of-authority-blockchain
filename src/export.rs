use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use crate::blockchain::Blockchain;

/// Writes one line per block to `path`, creating parent folders if needed.
pub fn write_report(chain: &Blockchain, path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    for block in chain.blocks() {
        writeln!(writer, "{}", block)?;
    }
    // BufWriter swallows errors on drop, so flush explicitly
    writer.flush()?;
    Ok(())
}

/// Counts the lines in a report file.
pub fn count_report_lines(path: &Path) -> io::Result<usize> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut count = 0;
    for line in reader.lines() {
        line?; // each line can fail on its own
        count += 1;
    }
    Ok(count)
}

/// Adds one summary line to the END of an existing report.
/// Fails if the file does not exist, because `create` is not set.
pub fn append_summary(chain: &Blockchain, path: &Path) -> io::Result<()> {
    let mut file = OpenOptions::new().append(true).open(path)?;
    writeln!(
        file,
        "summary: {} blocks, {} txs",
        chain.len(),
        chain.transaction_count()
    )?;
    Ok(())
}

/// Size of a file in bytes, read from its metadata.
pub fn file_size(path: &Path) -> io::Result<u64> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Blockchain, ValidatorSet, WorldState};

    fn small_chain() -> Blockchain {
        let validators = ValidatorSet::new(vec!["v0".to_string()]).unwrap();
        let mut chain = Blockchain::new(WorldState::new(), validators, 1000, 0).unwrap();
        chain.add_block("v0", 1000, Vec::new()).unwrap();
        chain
    }

    /// A scratch folder unique per test, so parallel tests never collide
    /// and remove_dir_all is safe.
    fn scratch_dir(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("poa-{}-{}", label, std::process::id()))
    }

    #[test]
    fn report_has_one_line_per_block() {
        let dir = scratch_dir("report");
        let path = dir.join("nested").join("report.txt");

        let chain = small_chain();
        write_report(&chain, &path).unwrap();
        assert_eq!(count_report_lines(&path).unwrap(), 2); // genesis + 1

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn missing_file_is_an_error() {
        let path = std::env::temp_dir().join("poa-definitely-not-here.txt");
        assert!(count_report_lines(&path).is_err());
    }

    #[test]
    fn append_adds_a_line_at_the_end() {
        let dir = scratch_dir("append");
        let path = dir.join("report.txt");

        let chain = small_chain();
        write_report(&chain, &path).unwrap();
        append_summary(&chain, &path).unwrap();

        assert_eq!(count_report_lines(&path).unwrap(), 3);
        let text = fs::read_to_string(&path).unwrap();
        assert!(
            text.lines()
                .last()
                .unwrap()
                .starts_with("summary: 2 blocks, 0 txs")
        );

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn append_does_not_create_a_missing_file() {
        let path = scratch_dir("no-create").join("report.txt");
        let chain = small_chain();
        assert!(append_summary(&chain, &path).is_err());
    }

    #[test]
    fn file_size_matches_the_bytes_written() {
        let dir = scratch_dir("size");
        let path = dir.join("report.txt");

        write_report(&small_chain(), &path).unwrap();
        let on_disk = fs::read(&path).unwrap();
        assert_eq!(file_size(&path).unwrap(), on_disk.len() as u64);

        fs::remove_dir_all(&dir).unwrap();
    }
}
