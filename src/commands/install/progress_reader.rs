use indicatif::ProgressBar;
use std::io::Read;

pub struct ProgressReader<'a, R: Read> {
    reader: R,
    progress_bar: &'a ProgressBar,
}
impl<'a, R: Read> ProgressReader<'a, R> {
    pub fn new(reader: R, progress_bar: &'a ProgressBar) -> Self {
        Self {
            reader,
            progress_bar,
        }
    }
}
impl<'a, R: Read> Read for ProgressReader<'a, R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let read = self.reader.read(buf)?;
        self.progress_bar.inc(read as u64);
        Ok(read)
    }
}
