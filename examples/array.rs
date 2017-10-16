// Arrays and optionals

// Integer delta
struct Delta {
    dx: i8,
    dy: i8
}

// Command for a player is either Some delta or None
type Command = Option<Delta>;

// Aggregating type synonyms inside struct
struct CommandSet {
    current: u8,
    commands: [Command; 64]
}
