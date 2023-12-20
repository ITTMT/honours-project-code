pub enum Token {
    Ampersand,  // (&) Switch to character reference in data state
    LPointy,    // (<) Tag open state
    Error,      // Parse Error


}

// I will re-consider making my own lexer, if the time comes for it. Because HTML can technically store both JS and CSS
// in it which will be a lot of work. HTML5Tokenizer crate exists, and seems to do what is necessary,
// delete this later if it comes to it.