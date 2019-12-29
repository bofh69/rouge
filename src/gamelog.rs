pub struct GameLog {
    pub entries: Vec<String>,
}

impl GameLog {
    pub fn log<T: Into<String>>(&mut self, s: T) {
        self.entries.push(s.into());
    }
}
