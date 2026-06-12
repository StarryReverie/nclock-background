#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

impl<T> Size<T> {
    pub fn new(width: T, height: T) -> Self {
        Self { width, height }
    }

    pub fn convert_with<F, U>(self, conv: F) -> Size<U>
    where
        F: Fn(T) -> U,
    {
        Size::new(conv(self.width), conv(self.height))
    }
}
