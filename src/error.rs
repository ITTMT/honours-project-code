pub struct Error {
    pub error_string: String,
    pub error_occurred: bool
}

impl Error {
    pub fn new() -> Error {
        Error {
            error_string: String::new(),
            error_occurred: false
        }
    }

	pub fn handle_error(&mut self, error_message: String) {
		self.error_string = error_message;
		self.error_occurred = true;
	} 

	
}