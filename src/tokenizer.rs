use std::collections::HashMap;
use utils::*;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct TokenType(pub u32);

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct State(pub u32);

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
        pub value: Vec<u8>,
        pub t: TokenType,
        pub s: State
}

pub struct Tokenizer {
        token_map: HashMap<u8, TokenType>,
        transition_map: HashMap<State, HashMap<TokenType, State>>
}

impl Tokenizer {
        pub fn new(token_map: &str, transition_map: &str) -> Self {
                Tokenizer {
                        token_map: Tokenizer::compile_tokens_ascii(token_map),
                        transition_map: Tokenizer::compile_states(transition_map)
                }
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
        pub fn tokenize(&mut self, token_str: &[u8]) -> Vec<Token> {
                let mut tokens: Vec<Token> = Vec::new();

                let mut curr_token: Vec<u8> = Vec::new();
                let mut raw_bytes = token_str.to_vec();
                let mut curr_state = State(utils::get_hash_val(b"Start"));
                let mut last_byte_t = TokenType(0);

                while !raw_bytes.is_empty() {
                        //println!("Byte: {:?}", raw_bytes[0]);
                        //println!("curr_token: {:?}", curr_token);
                        //println!("curr_state: {:?}", curr_state);
                        //println!("tokens: {:?}", tokens);    

                        let curr_byte = raw_bytes[0];
                        let curr_byte_t = match self.token_map.get(&curr_byte) {
                                None => TokenType(0),
                                Some(v) => v.to_owned()
                        };

                        let new_state = match self.transition_map.get(&curr_state).unwrap().get(&curr_byte_t) {
                                None => {
                                        let start_state = State(utils::get_hash_val(b"Start"));
                                        let mut s = &curr_state;
                                        if curr_byte_t != TokenType(0) {
                                                tokens.push(Token { 
                                                        t: last_byte_t.to_owned(), 
                                                        s: curr_state.to_owned(),
                                                        value: curr_token.to_owned()});
                                                curr_token.clear();                                          
                                                s = &start_state;
                                        } else {
                                                raw_bytes.remove(0);
                                        };
                                        s.to_owned()
                                },
                                Some(v) => {
                                        if v == &curr_state && curr_state != State(utils::get_hash_val(b"Start")) {
                                                curr_token.push(raw_bytes.remove(0));
                                        }
                                        if v != &curr_state && curr_state != State(utils::get_hash_val(b"Start")) {
                                                tokens.push(Token { 
                                                        t: last_byte_t.to_owned(), 
                                                        s: curr_state.to_owned(),
                                                        value: curr_token.to_owned()});
                                                curr_token.clear();
                                        };
                                        v.to_owned()
                                } 
                        };

                        if raw_bytes.is_empty() {
                                tokens.push(Token { 
                                        t: curr_byte_t.to_owned(), 
                                        s: new_state.to_owned(),
                                        value: curr_token.to_owned() });
                        }

                        last_byte_t = curr_byte_t;
                        curr_state = new_state;
                };
                tokens
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

                let transition_map: HashMap<State, HashMap<TokenType, State>> = Tokenizer::compile_states(&TRANSITIONS);

                let test_state_alpha = State(utils::get_hash_val(b"Alpha"));
                let test_token_type_alpha = TokenType(utils::get_hash_val(b"Alpha"));

                assert_eq!(transition_map[&test_state_alpha][&test_token_type_alpha], test_state_alpha);
        }


        #[test]
        fn test_compile_tokens() {

                let token_map: HashMap<u8, TokenType> = Tokenizer::compile_tokens_ascii(&TOKENS);
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

                assert_eq!(tokenized.len(), 110);
        }
}

