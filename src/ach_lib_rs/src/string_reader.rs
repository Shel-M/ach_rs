#[derive(Debug)]
pub struct StringReader {
    inner: String,
    pos: usize,
}

impl StringReader {
    pub fn new(inner: String) -> Self {
        StringReader { inner, pos: 0 }
    }

    /// Read a number of chars from the contained String.
    /// Will not panic if reading past end, instead returning an the remainder of the String.
    /// If reading after reaching the end, the result is an empty String
    pub fn read(&mut self, size: usize) -> String {
        let pos = self.pos;
        let len = self.len();
        let mut inner_read = |size| {
            let val = self.inner[self.pos..self.pos + size].to_string();
            self.pos += size;
            val
        };

        if size == 0 {
            // If trying to read a 0-length string, just return ""
            return "".to_string();
        } else if size + pos > len {
            // If trying to read beyond the length of self, read to end.
            return inner_read(len - pos);
        }

        inner_read(size)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[allow(unused)]
    pub fn seek(&mut self, pos: usize) {
        if pos > self.len() {
            self.pos = self.len();
            return;
        }
        self.pos = pos;
    }
}

#[cfg(test)]
mod string_reader_tests {
    use crate::string_reader::StringReader;

    const LOREM: &str = "lorem ipsum dolor sit amet";

    #[test]
    fn test_read() {
        let mut subject = StringReader::new(LOREM.to_string());
        println!("left: {}", LOREM);
        println!("right: {:?}", subject);
        let res = subject.read(LOREM.len());
        assert_eq!(LOREM, res);
        println!("left: {}", LOREM.len());
        println!("right (after): {:?}", subject);
        assert_eq!(LOREM.len(), subject.pos)
    }

    #[test]
    fn test_read_twice() {
        let mut subject = StringReader::new(LOREM.to_string());
        println!("left: {}", LOREM);
        println!("right: {:?}", subject);
        let res = subject.read(LOREM.len() / 2);
        assert_eq!(LOREM[0..13], res);
        println!("First read complete, performing second.");
        println!("left: {}", LOREM.len());
        println!("right: {:?}", subject);
        assert_eq!(LOREM[13..LOREM.len()], subject.read(31));
        println!("right (after): {:?}", subject)
    }

    #[test]
    fn test_read_past_end() {
        let mut subject = StringReader::new(LOREM.to_string());
        println!("left: {}", LOREM);
        println!("right: {:?}", subject);
        println!("right len: {}", subject.len());
        let res = subject.read(LOREM.len() + 5);
        assert_eq!(LOREM, res);
        println!("left: {}", LOREM.len());
        println!("right (after): {:?}", subject);
        assert_eq!(LOREM.len(), subject.pos)
    }

    #[test]
    fn test_read_at_end() {
        let mut subject = StringReader {
            inner: LOREM.to_string(),
            pos: LOREM.len(),
        };
        let res = subject.read(1);
        println!("left is an empty String");
        println!("right: {:?}", res);
        assert_eq!("", res)
    }

    #[test]
    fn test_len() {
        let subject = StringReader::new(LOREM.to_string());
        println!("left: {}", LOREM.len());
        println!("right: {}", subject.len());
        assert_eq!(LOREM.len(), subject.len())
    }

    #[test]
    fn test_seek() {
        let mut subject = StringReader::new(LOREM.to_string());
        let compare = StringReader {
            inner: LOREM.to_string(),
            pos: 15,
        };
        subject.seek(15);
        println!("left: {:?}", compare);
        println!("right: {:?}", subject);
        assert_eq!(format!("{:?}", compare), format!("{:?}", subject))
    }
}
