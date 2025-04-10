struct StringGenerator {
    count: usize,
    n_of_pairs: usize,
    max_key_len: usize,
    max_value_len: usize,
    delimiter: u8,
}

impl StringGenerator {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz";

    fn new(n: usize, key_len: usize, value_len: usize, delim: u8) -> Self {
        Self {
            count: 0,
            n_of_pairs: n,
            max_key_len: key_len,
            max_value_len: value_len,
            delimiter: delim,
        }
    }

    fn random_string_gen(&self, len: usize) -> Vec<u8> {
        //https://rust-lang-nursery.github.io/rust-cookbook/algorithms/randomness.html#create-random-passwords-from-a-set-of-alphanumeric-characters
        return thread_rng()
            .sample_iter(&Alphanumeric)
            .take(len)
            .map(u8::from)
            .collect();
    }
}

impl Iterator for StringGenerator {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count < self.n_of_pairs {
            //final random pair
            let mut pair: Vec<u8> = Vec::with_capacity(self.max_value_len + self.max_key_len + 2);
            pair.push(self.delimiter);

            //random key
            let mut rand_key = self.random_string_gen(self.max_key_len);
            pair.append(&mut rand_key);

            //delimiter
            pair.push(self.delimiter);

            //random value
            let mut rand_value = self.random_string_gen(self.max_value_len);
            pair.append(&mut rand_value);

            //println!(
            //    "random pair generated: {}",
            //    std::str::from_utf8(&pair).unwrap()
            //);

            self.count += 1;

            Some(pair)
        } else {
            None
        }
    }
}

struct FileReader<'a> {
    //this will contain a line
    buff: Vec<u8>,
    reader: Option<io::BufReader<File>>,
    filename: &'a str,
}

impl<'a> FileReader<'a> {
    fn new(f: &'a str) -> Self {
        Self {
            buff: Vec::new(),
            reader: None,
            filename: f,
        }
    }

    fn read_file(&mut self) {
        let file = File::open(self.filename).expect("couldn't open given file");
        self.reader = Some(io::BufReader::new(file));
    }
}

//impl<'a> Iterator for FileReader<'a> {
//    type Item = &'a [u8];
//
//    fn next(&mut self) -> Option<Self::Item> {
//        self.buff.clear();
//        let x = self.reader?.read_until(0x0A, &mut self.buff);
//        match x {
//            Ok(_) => Some(&self.buff),
//            _ => None,
//        }
//    }
//}
