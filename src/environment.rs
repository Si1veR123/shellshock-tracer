enum GameMode {
    Deathmatch,
    Points,
    Assassin,
    Juggernaut,
    Rebound,
    Charge,
    Marksman,
    Shoccer,
    Vortex,
    Unknown
}

enum Wind {
    Left(u8),
    Right(u8)
}

pub struct Environment {
    mode: GameMode,
    wind: Wind
}
