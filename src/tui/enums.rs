#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    StarsDesc,
    StarsAsc,
    ScoreDesc,
    ScoreAsc,
    NameAsc,
    NameDesc,
}

impl SortMode {
    pub fn cycle(self) -> Self {
        match self {
            SortMode::StarsDesc => SortMode::StarsAsc,
            SortMode::StarsAsc => SortMode::ScoreDesc,
            SortMode::ScoreDesc => SortMode::ScoreAsc,
            SortMode::ScoreAsc => SortMode::NameAsc,
            SortMode::NameAsc => SortMode::NameDesc,
            SortMode::NameDesc => SortMode::StarsDesc,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            SortMode::StarsDesc => "Stars ↓",
            SortMode::StarsAsc => "Stars ↑",
            SortMode::ScoreDesc => "Score ↓",
            SortMode::ScoreAsc => "Score ↑",
            SortMode::NameAsc => "Name A-Z",
            SortMode::NameDesc => "Name Z-A",
        }
    }
}

impl Default for SortMode {
    fn default() -> Self {
        SortMode::StarsDesc
    }
}
