// Generated from vendor/codewhale-design/tokens.json 1.1.1 by tests/generated.rs.
// Do not edit. Regenerate: CODEWHALE_BLESS=1 cargo test --test generated

/// The design tokens version these roles were generated from.
pub const TOKENS_VERSION: &str = "1.1.1";

/// What a color is for. Components name roles; they never name colors.
/// Names follow `tokens.json`, so one vocabulary covers the desktop app,
/// the web and the terminal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Role {
    /// The deepest ground: rails and things put away.
    Sidebar,
    /// The stage.
    Background,
    /// A raised panel.
    Surface,
    /// A row under the pointer.
    Hover,
    /// The selected row.
    Selected,
    /// Body ink.
    Foreground,
    /// Ink that recedes: details, verbs in hints, asides.
    Muted,
    /// Quiet lines that separate.
    Border,
    /// Edges that identify a control or a decision.
    BorderStrong,
    /// Actions, focus and links.
    Primary,
    /// Ink on a `Primary` ground.
    PrimaryForeground,
    /// Work happening now, and work done.
    Live,
    /// Needs you.
    Attention,
    /// Failed or destructive.
    Danger,
}

impl Role {
    pub const COUNT: usize = 14;
    pub const ALL: [Role; Self::COUNT] = [
        Role::Sidebar,
        Role::Background,
        Role::Surface,
        Role::Hover,
        Role::Selected,
        Role::Foreground,
        Role::Muted,
        Role::Border,
        Role::BorderStrong,
        Role::Primary,
        Role::PrimaryForeground,
        Role::Live,
        Role::Attention,
        Role::Danger,
    ];

    /// The `tokens.json` key this role reads.
    #[must_use]
    pub const fn token_name(self) -> &'static str {
        match self {
            Role::Sidebar => "sidebar",
            Role::Background => "background",
            Role::Surface => "surface",
            Role::Hover => "hover",
            Role::Selected => "selected",
            Role::Foreground => "foreground",
            Role::Muted => "muted_foreground",
            Role::Border => "border",
            Role::BorderStrong => "border_strong",
            Role::Primary => "primary",
            Role::PrimaryForeground => "primary_foreground",
            Role::Live => "live",
            Role::Attention => "attention",
            Role::Danger => "danger",
        }
    }

    /// Position in [`Role::ALL`] and in the color tables.
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }
}

/// dark token colors, `0xRRGGBB`, indexed by [`Role::index`].
pub(crate) const DARK: [u32; Role::COUNT] = [
    0x191a1c, // Sidebar
    0x202123, // Background
    0x2a2b2e, // Surface
    0x303134, // Hover
    0x37393d, // Selected
    0xefeeeb, // Foreground
    0xb1b1ad, // Muted
    0x3b3c3f, // Border
    0x828386, // BorderStrong
    0x90b9ff, // Primary
    0x15243e, // PrimaryForeground
    0x9ec7b2, // Live
    0xe8b077, // Attention
    0xe39a90, // Danger
];

/// dark xterm 256-color indices (16..=255 only; 0..=15 belong to the
/// user's profile). Nearest index, except where contrast needed a move:
/// - dark danger: 174 -> 210 (nearest failed its contrast floor)
pub(crate) const DARK_256: [u8; Role::COUNT] = [
    234, // Sidebar
    235, // Background
    236, // Surface
    236, // Hover
    237, // Selected
    255, // Foreground
    145, // Muted
    237, // Border
    102, // BorderStrong
    111, // Primary
    235, // PrimaryForeground
    151, // Live
    180, // Attention
    210, // Danger
];

/// light token colors, `0xRRGGBB`, indexed by [`Role::index`].
pub(crate) const LIGHT: [u32; Role::COUNT] = [
    0xf0ede8, // Sidebar
    0xfaf8f5, // Background
    0xffffff, // Surface
    0xe8e5e0, // Hover
    0xdfdcd6, // Selected
    0x28292b, // Foreground
    0x5f605d, // Muted
    0xd9d5cf, // Border
    0x807c76, // BorderStrong
    0x245bc7, // Primary
    0xfbf5ee, // PrimaryForeground
    0x3a6352, // Live
    0x86520d, // Attention
    0x9e3f36, // Danger
];

/// light xterm 256-color indices (16..=255 only; 0..=15 belong to the
/// user's profile). Nearest index, except where contrast needed a move:
/// - light primary: 26 -> 25 (nearest failed its contrast floor)
/// - light live: 239 -> 23 (nearest lost its hue)
/// - light attention: 94 -> 58 (nearest failed its contrast floor)
/// - light danger: 131 -> 124 (nearest failed its contrast floor)
/// - light border_strong: 244 -> 243 (nearest failed its contrast floor)
pub(crate) const LIGHT_256: [u8; Role::COUNT] = [
    255, // Sidebar
    231, // Background
    231, // Surface
    254, // Hover
    253, // Selected
    235, // Foreground
    59, // Muted
    188, // Border
    243, // BorderStrong
    25, // Primary
    255, // PrimaryForeground
    23, // Live
    58, // Attention
    124, // Danger
];

/// The blue ombre (`Ground::Ocean`): the dark table with its grounds and
/// quiet line tinted 0.5 toward `LOGO_BOTTOM` at their own luminance,
/// so every contrast floor holds. Truecolor only; 256 colors use `DARK_256`.
pub(crate) const OCEAN: [u32; Role::COUNT] = [
    0x06183c, // Sidebar
    0x0a1f47, // Background
    0x122957, // Surface
    0x162f60, // Hover
    0x1b376b, // Selected
    0xefeeeb, // Foreground
    0xb1b1ad, // Muted
    0x1e3a70, // Border
    0x828386, // BorderStrong
    0x90b9ff, // Primary
    0x15243e, // PrimaryForeground
    0x9ec7b2, // Live
    0xe8b077, // Attention
    0xe39a90, // Danger
];
