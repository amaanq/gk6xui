#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Layer {
    Invalid = 0,
    Base = 1,
    Layer1 = 2,
    Layer2 = 3,
    Layer3 = 4,
    Driver = 5,
}
