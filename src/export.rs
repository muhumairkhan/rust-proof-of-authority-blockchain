use std::fs::{self, File};
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

    #[test]
    fn report_has_one_line_per_block() {
        // a scratch folder, so remove_dir_all is safe
        let dir = std::env::temp_dir().join(format!("poa-report-test-{}", std::process::id()));
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
}
