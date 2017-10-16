// Quantization and bounding of integers

// Inline quantization attributes for floats
struct YawDistance {
    // Wrapping around
    #[qwrap(low="0", high="6.28318", step="0.01")]
    yaw: f32,

    // Linear interval
    #[qlinspace(low="0", high="1e4", step="0.001")]
    distance: f32
}

// Using type synonym
#[qlinspace(low="0", high="1e4", step="0.001")]
type Time = f32;

// Bounding integers
#[ibound("1", "10")]
type PlayerID = u8;

// Parameters to attribute key-value pairs use Python-style
// keyword arguments and defaults.
#[qlinspace("100", "200", step="1.")]
type QuantExampleTwo = f64;
