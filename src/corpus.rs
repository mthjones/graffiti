use std::collections::HashMap;
use std::io::Read;
use std::fs;
use utils::*;

use scanner::*;
use tokenizer::*;

pub struct Corpus<R: Read> {
    scanners: Vec<Scanner<R>>,
    tokenizer: Tokenizer
}

impl<R: Read> Corpus<R> {
    pub fn new(readers: Vec<R>, tokenizer: Tokenizer) -> Self {
        let scanners = readers.into_iter().map(Scanner::new).collect();

        Corpus {
            scanners: scanners,
            tokenizer: tokenizer
        }
    }
    pub fn get_scanners(&self) -> &Vec<Scanner<R>> {
        &self.scanners
    }
    pub fn words(&mut self, pos: usize) -> Vec<Vec<u8>> {
        let contents = self.scanners[pos].scan().unwrap();

        let tokens = self.tokenizer.tokenize(&contents);

        let mut filtered_tokens = tokens.into_iter().filter(|f| f.s == State(utils::get_hash_val(&String::from("Alpha").into_bytes()))).collect::<Vec<Token>>();
        filtered_tokens.into_iter().map(|t| t.value).collect::<Vec<Vec<u8>>>()        
    }
    pub fn allwords(&mut self) -> Vec<Vec<u8>> {
        let mut all_tokens: Vec<Vec<u8>> = Vec::new();
        for s in self.scanners.iter_mut() {
            let contents = s.scan().unwrap();
            let tokens = self.tokenizer.tokenize(&contents);

            let filtered_tokens = tokens.into_iter().filter(|f| f.s == State(utils::get_hash_val(&String::from("Alpha").into_bytes()))).collect::<Vec<Token>>();
            all_tokens.append(&mut filtered_tokens.into_iter().map(|t| t.value).collect::<Vec<Vec<u8>>>());
        };
        all_tokens
    }

}


#[cfg(test)]
mod tests {

    use std::fs;

    use super::*;
    use scanner::*;
    use tokenizer::*;

    static tokens: &'static str = 
            "
            Alpha => 65..123
            Number => 48..57
            Whitespace => 9,10,13,32
            Punctuation => 33..46
            Punctuation => 58..65
            Slash => 47
            ";

    static transitions: &'static str = 
            "
            Start => Alpha => Alpha
            Start => Number => Number
            Start => Whitespace => Whitespace
            Start => Punctuation => Punctuation
            Start => Slash => Slash
            Slash => Slash => Slash
            Slash => Whitespace => Whitespace
            Slash => Alpha => Pos
            Alpha => Alpha | Number => Alpha
            Pos => Alpha => Pos
            Number => Number => Number
            Number => Alpha => Alpha
            Whitespace => Whitespace => Whitespace
            Punctuation => Punctuation => Punctuation
            ";

    fn brown(tokenizer: Tokenizer) -> Corpus<fs::File> {
        let files = fs::read_dir("/brown/").unwrap().map(|f| fs::File::open(f.unwrap().path()).unwrap()).collect();

        Corpus::new(files, tokenizer)
    }

    #[test]
    fn test_get_files() {
        let mut tokenizer = Tokenizer::new(&tokens, &transitions);
        let mut brown_corpus = brown(tokenizer);

        let scanners = brown_corpus.get_scanners();

        assert_eq!(scanners.len(), 504);
    }

    #[test]
    fn test_get_words() {
        let mut tokenizer = Tokenizer::new(&tokens, &transitions);
        let mut brown_corpus = brown(tokenizer);

        let words = brown_corpus.words(0);
        let num_words = words.len();

        println!("{:?}", &words.into_iter().map(|w_v| String::from_utf8(w_v).unwrap()).collect::<Vec<String>>());
        println!("{:?}", num_words);

        assert!(num_words == 2088);
    }
}