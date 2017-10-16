// Structures of known size, containing other structures and primitive types.

struct Inner {
    int_field: i32,
    string_field: String
}

struct Outer {
    direct_field: u8,
    inner: Inner
}
