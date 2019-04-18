pub trait VersionLimit {
    fn within_limits(&self) -> bool;
}

impl VersionLimit for u32 {
    fn within_limits(&self) -> bool {
        *self > 0 && *self < 2_147_483_648
    }
}
