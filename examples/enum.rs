// C-style enums are also permitted

enum Kind {
    Apple,
    Banana,
    Grapefruit
}

struct EnumBearingStruct {
    // Enum is represented as an integer
    kind: Kind,
    // Strings are represented as-is
    name: String
}
