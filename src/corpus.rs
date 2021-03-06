use std::fs;
use utils::*;

use scanner::*;
use tokenizer::*;

pub struct Corpus {
    scanners: Vec<Scanner>,
    tokenizer: Tokenizer
}

impl Corpus {
    pub fn new(root_path: &str, tokenizer: Tokenizer) -> Self {
        let scanners: Vec<Scanner> = Corpus::register_scanners(root_path);

        Corpus {
            scanners: scanners,
            tokenizer: tokenizer
        }
    }
    pub fn brown() -> Self {
        let tokens: &'static str = 
            "
            Alpha => 65..123
            Number => 48..57
            Whitespace => 9,10,13,32
            Punctuation => 33..46
            Punctuation => 58..65
            Slash => 47
            ";

        let transitions: &'static str = 
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

        let tokenizer = Tokenizer::new(&tokens, &transitions);
        let brown_path: &'static str = "/brown/";

        let scanners: Vec<Scanner> = Corpus::register_scanners(brown_path);

        Corpus {
            scanners: scanners,
            tokenizer: tokenizer
        }
     
    }
    fn register_scanners(root_path: &str) -> Vec<Scanner> {
        let paths = fs::read_dir(root_path).unwrap().map(|f| f.unwrap().file_name().into_string().unwrap()).collect::<Vec<String>>();
        paths.iter().map(|f| Scanner::new(&format!("{}{}", root_path, f))).collect::<Vec<Scanner>>()
    }
    pub fn get_scanners(&self) -> &Vec<Scanner> {
        &self.scanners
    }
    pub fn words(&mut self, pos: usize) -> Vec<Vec<u8>> {
        let contents = self.scanners[pos].scan().unwrap();

        let tokens = self.tokenizer.tokenize(&contents);

        let filtered_tokens = tokens.into_iter().filter(|f| f.s == State(utils::get_hash_val(b"Alpha"))).collect::<Vec<Token>>();
        filtered_tokens.into_iter().map(|t| t.value).collect::<Vec<Vec<u8>>>()        
    }
    pub fn allwords(&mut self) -> Vec<Vec<u8>> {
        let mut all_tokens: Vec<Vec<u8>> = Vec::new();
        for s in &self.scanners {
            let contents = s.scan().unwrap();
            let tokens = self.tokenizer.tokenize(&contents);

            let filtered_tokens = tokens.into_iter().filter(|f| f.s == State(utils::get_hash_val(b"Alpha"))).collect::<Vec<Token>>();
            all_tokens.append(&mut filtered_tokens.into_iter().map(|t| t.value).collect::<Vec<Vec<u8>>>());
        };
        all_tokens
    }

}


#[cfg(test)]
mod tests {

    use std::str;
    use super::*;
    use tokenizer::*;

    static TOKENS: &'static str = 
            "
            Alpha => 65..123
            Number => 48..57
            Whitespace => 9,10,13,32
            Punctuation => 33..46
            Punctuation => 58..65
            Slash => 47
            ";

    static TRANSITIONS: &'static str = 
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

    #[test]
    fn test_get_files() {
        let tokenizer = Tokenizer::new(&TOKENS, &TRANSITIONS);

        let brown_corpus = Corpus::new("/brown/", tokenizer);

        let scanners = brown_corpus.get_scanners();

        assert_eq!(scanners.len(), 504);
    }

    #[test]
    fn test_get_words() {
        let tokenizer = Tokenizer::new(&TOKENS, &TRANSITIONS);

        let mut brown_corpus = Corpus::new("/brown/", tokenizer);

        let words = brown_corpus.words(0);
        let num_words = words.len();

        println!("{:?}", &words.into_iter().map(|w_v| String::from_utf8(w_v).unwrap()).collect::<Vec<String>>());
        println!("{:?}", num_words);

        assert!(num_words == 2088);
    }
}