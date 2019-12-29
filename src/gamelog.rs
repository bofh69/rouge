pub struct GameLog {
    pub entries: Vec<String>,
}

impl GameLog {
    pub fn log(&mut self, s: &str) {
        self.entries.push(s.to_string());
    }
}
