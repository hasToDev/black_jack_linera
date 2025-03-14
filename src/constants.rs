/// Spades:
/// 1 = Ace, 11 = Jack, 12 = Queen, 13 = King
///
/// Hearts:
/// 14 = Ace, 24 = Jack, 25 = Queen, 26 = King
///
/// Diamonds:
/// 27 = Ace, 37 = Jack, 38 = Queen, 39 = King
///
/// Clubs:
/// 40 = Ace, 50 = Jack, 51 = Queen, 52 = King
pub const CARD_DECKS: [u8; 52] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13,
    14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39,
    40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52
];

/// ------------------------------------------------------------------------------------------
pub const MILLENNIUM: u64 = 946684800000000;
pub const UNIX_MICRO_IN_18_SECONDS: u64 = 18_000_000;
pub const UNIX_MICRO_IN_10_SECONDS: u64 = 10_000_000;