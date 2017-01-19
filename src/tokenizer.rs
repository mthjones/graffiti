use std::collections::HashMap;
use utils::*;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub struct TokenType(pub u32);

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub struct State(pub u32);

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
        pub value: Vec<u8>,
        pub t: TokenType,
        pub s: State
}

pub fn compile_states(transition_map: &str) -> HashMap<State, HashMap<TokenType, State>> {
        let mut parsed_trans = HashMap::new();
        for t in transition_map.lines() {
                let v: Vec<&str> = t.split("=>").map(|s| s.trim()).collect();
                if v[0].is_empty() {
                        continue;
                }
                let v_bytes: Vec<u8> = v[0].bytes().collect();
                let start_state: State = State(utils::get_hash_val(&v_bytes));
                let tokens: Vec<&str> = v[1].split("|").map(|t| t.trim()).collect();
                let v2_bytes: Vec<u8> = v[2].bytes().collect();
                let end_state: State = State(utils::get_hash_val(&v2_bytes));

                let x = parsed_trans.entry(start_state).or_insert(HashMap::new());
                for t in tokens {
                        let t_bytes: Vec<u8> = t.bytes().collect();
                        let token_type = TokenType(utils::get_hash_val(&t_bytes));
                        (*x).entry(token_type).or_insert(end_state.to_owned());
                }
        };
        parsed_trans
}

pub fn compile_tokens_ascii(tokens: &str) -> HashMap<u8, TokenType> {
        let mut token_map = HashMap::new();
        for t in tokens.lines() {
                let v: Vec<&str> = t.split("=>").map(|s| s.trim()).collect();
                if v[0].is_empty() {
                        continue;
                }
                let mut vals: Vec<u8> = Vec::new();
                if v[1].contains("..") {
                        let bounds: Vec<u8> = v[1].split("..").map(|c| c.parse::<u8>().unwrap()).collect();
                        vals.append(&mut (bounds[0]..bounds[1]).collect());
                } else if v[1].contains(',') {
                        let mut nums: Vec<u8> = v[1].split(',').map(|i| i.parse::<u8>().unwrap()).collect();
                        vals.append(&mut nums);
                } else {
                        let single_num = v[1].parse::<u8>().unwrap(); 
                        vals.push(single_num);
                }
                let t_bytes: Vec<u8> = v[0].bytes().collect();
                for u in vals {
                        token_map.insert(u, TokenType(utils::get_hash_val(&t_bytes)));
                }
        };
        token_map
}

pub struct Tokenizer {
        token_map: HashMap<u8, TokenType>,
        transition_map: HashMap<State, HashMap<TokenType, State>>
}

impl Tokenizer {
        pub fn new(token_map: &str, transition_map: &str) -> Self {
                Tokenizer {
                        token_map: compile_tokens_ascii(token_map),
                        transition_map: compile_states(transition_map)
                }
        }

        pub fn tokenize(&mut self, token_str: &[u8]) -> Vec<Token> {
                let stream = TokenStream::new(token_str, &self.token_map, &self.transition_map);

                stream.collect()
        }
}

pub struct TokenStream<'a, 'b> {
        bytes: &'a [u8],
        current_token_span: (usize, usize),
        current_type: TokenType,
        current_state: State,
        token_map: &'b HashMap<u8, TokenType>,
        transition_map: &'b HashMap<State, HashMap<TokenType, State>>
}

impl<'a, 'b> TokenStream<'a, 'b> {
        pub fn new(bytes: &'a [u8], token_map: &'b HashMap<u8, TokenType>, transition_map: &'b HashMap<State, HashMap<TokenType, State>>) -> Self {
                TokenStream {
                        bytes: bytes,
                        current_token_span: (0, 0),
                        current_type: TokenType(0),
                        current_state: State(utils::get_hash_val(b"Start")),
                        token_map: token_map,
                        transition_map: transition_map
                }
        }

        fn get_next_state(&self, token_type: TokenType) -> State {
                match self.transition_map.get(&self.current_state).unwrap().get(&token_type) {
                        Some(&v) => v,
                        None => State(utils::get_hash_val(b"Start"))
                }
        }

        fn complete_token(&mut self) -> Token {
                let (token_start, next_pos) = self.current_token_span;
                self.current_token_span = (next_pos, next_pos);
                Token {
                        t: self.current_type,
                        s: self.current_state,
                        value: self.bytes[token_start..next_pos].to_vec()
                }
        }
}

impl<'a, 'b> Iterator for TokenStream<'a, 'b> {
        type Item = Token;

        fn next(&mut self) -> Option<Self::Item> {
                loop {
                        let (token_start, next_pos) = self.current_token_span;
                        if next_pos == self.bytes.len() && token_start == next_pos {
                                return None;
                        } else if next_pos == self.bytes.len() {
                                return Some(self.complete_token());
                        }
                        
                        let next_byte = self.bytes[next_pos];
                        if let Some(&next_token_type) = self.token_map.get(&next_byte) {
                                let next_state = self.get_next_state(next_token_type);
                                if self.current_state != next_state && self.current_state != State(utils::get_hash_val(b"Start")) {
                                        let token = self.complete_token();
                                        self.current_state = next_state;
                                        self.current_type = next_token_type;
                                        return Some(token);
                                }
                                if self.current_state == State(utils::get_hash_val(b"Start")) {
                                        self.current_state = next_state;
                                        self.current_type = next_token_type;
                                }
                        }

                        self.current_token_span = (token_start, next_pos + 1);
                }
        }
}

#[cfg(test)]
mod tests {

        use super::*;
        use utils::*;
        use std::collections::HashMap;


        static TOKENS: &'static str = 
                "
                Alpha => 65..91
                Alpha => 97..123
                Number => 48..57
                Whitespace => 9,10,13,32
                Punctuation => 33..47
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

        static BROWN_CA01: &'static str = "

                The/at Fulton/np-tl County/nn-tl Grand/jj-tl Jury/nn-tl said/vbd Friday/nr an/at investigation/nn of/in Atlanta's/np$ recent/jj primary/nn election/nn produced/vbd ``/`` no/at evidence/nn ''/'' that/cs any/dti irregularities/nns took/vbd place/nn ./.


        ";


        #[test]
        fn test_compile_states() {

                let transition_map: HashMap<State, HashMap<TokenType, State>> = compile_states(&TRANSITIONS);

                let test_state_alpha = State(utils::get_hash_val(b"Alpha"));
                let test_token_type_alpha = TokenType(utils::get_hash_val(b"Alpha"));

                assert_eq!(transition_map[&test_state_alpha][&test_token_type_alpha], test_state_alpha);
        }


        #[test]
        fn test_compile_tokens() {

                let token_map: HashMap<u8, TokenType> = compile_tokens_ascii(&TOKENS);
                let test_token_type_alpha = TokenType(utils::get_hash_val(b"Alpha"));

                assert_eq!(token_map[&65], test_token_type_alpha);
                assert_eq!(token_map[&122], test_token_type_alpha);
        }

        #[test]
        fn tokenize_test_1() {

                let mut tokenizer = Tokenizer::new(&TOKENS, &TRANSITIONS);

                let bs = vec![97, 98, 99];

                let tokenized = tokenizer.tokenize(&bs);
                let test_token = Token { 
                        t: TokenType(utils::get_hash_val(b"Alpha")),
                        s: State(utils::get_hash_val(b"Alpha")),
                        value: bs };

                assert_eq!(tokenized[0], test_token);
        }

        #[test]
        fn tokenize_test_2() {

                let bs = b"foo/bar";

                let mut hashed_dict: HashMap<u32, &str> = HashMap::new();
                hashed_dict.insert(utils::get_hash_val(b"Alpha"), "Alpha");
                hashed_dict.insert(utils::get_hash_val(b"Whitespace"), "Whitepace");
                hashed_dict.insert(utils::get_hash_val(b"Slash"), "Slash");
                hashed_dict.insert(utils::get_hash_val(b"Pos"), "Pos");

                println!("{:?}", hashed_dict);

                let test_alpha = Token {
                        t: TokenType(utils::get_hash_val(b"Alpha")),
                        s: State(utils::get_hash_val(b"Alpha")),
                        value: b"foo".to_vec() };
                let test_slash = Token {
                        t: TokenType(utils::get_hash_val(b"Slash")),
                        s: State(utils::get_hash_val(b"Slash")),
                        value: b"/".to_vec() };
                let test_pos = Token {
                        t: TokenType(utils::get_hash_val(b"Alpha")),
                        s: State(utils::get_hash_val(b"Pos")),
                        value: b"bar".to_vec() };

                let mut tokenizer = Tokenizer::new(&TOKENS, &TRANSITIONS);

                let tokenized = tokenizer.tokenize(bs);
                let test_tokens = vec![&test_alpha, &test_slash, &test_pos];

                println!("{:?}", &test_tokens);

                assert_eq!(tokenized.len(), 3);
                assert_eq!(tokenized[0], test_alpha);
                assert_eq!(tokenized[1], test_slash);
                assert_eq!(tokenized[2], test_pos);

        }

        #[test]
        fn tokenize_test_3() {

                let bs = b"foo ^ foo";

                let test_alpha = Token {
                        t: TokenType(utils::get_hash_val(b"Alpha")),
                        s: State(utils::get_hash_val(b"Alpha")),
                        value: b"foo".to_vec() };
                let test_white = Token {
                        t: TokenType(utils::get_hash_val(b"Whitespace")),
                        s: State(utils::get_hash_val(b"Whitespace")),
                        value: b"  ".to_vec() };

                let mut tokenizer = Tokenizer::new(&TOKENS, &TRANSITIONS);

                let tokenized = tokenizer.tokenize(bs);

                assert_eq!(tokenized.len(), 3);
                assert_eq!(tokenized[0], test_alpha);
                assert_eq!(tokenized[1], test_white);
                assert_eq!(tokenized[2], test_alpha);

        }

        #[test]
        fn tokenize_test_4() {
                let mut tokenizer = Tokenizer::new(&TOKENS, &TRANSITIONS);

                let tokenized = tokenizer.tokenize(&BROWN_CA01.as_bytes().to_vec());

                for (i, token) in tokenized.iter().enumerate() {
                        // println!("Tokenized[{}]: {:?}", i, token);
                }

                assert_eq!(tokenized.len(), 110);
        }
}

