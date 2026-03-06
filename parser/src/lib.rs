mod structures;
mod token;
mod test;

#[cfg(test)]
mod tests {
    use crate::token::Tokenizer;
    use super::*;

    #[test]
    fn test_tokens() {
        let input = " coucou cou cou";
        let mut t = Tokenizer::new(input);
        t.parse();
    }
}
