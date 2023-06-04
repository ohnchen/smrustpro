use super::World;

impl World {
    pub fn default() -> Self {
        Self {
            width: 42,
            height: 22,
            alive: vec![],
        }
    }

    pub fn pulsar() -> Self {
        Self {
            width: 42,
            height: 22,
            alive: vec![
                (9, 18),
                (9, 17),
                (9, 16),
                (9, 22),
                (9, 23),
                (9, 24),
                (11, 18),
                (11, 17),
                (11, 16),
                (11, 22),
                (11, 23),
                (11, 24),
                (8, 19),
                (7, 19),
                (6, 19),
                (12, 19),
                (13, 19),
                (14, 19),
                (8, 21),
                (7, 21),
                (6, 21),
                (12, 21),
                (13, 21),
                (14, 21),
                (4, 16),
                (4, 17),
                (4, 18),
                (4, 22),
                (4, 23),
                (4, 24),
                (16, 16),
                (16, 17),
                (16, 18),
                (16, 22),
                (16, 23),
                (16, 24),
                (8, 14),
                (7, 14),
                (6, 14),
                (12, 14),
                (13, 14),
                (14, 14),
                (8, 26),
                (7, 26),
                (6, 26),
                (12, 26),
                (13, 26),
                (14, 26),
            ],
        }
    }
}
