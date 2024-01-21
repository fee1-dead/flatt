#[flatt(embeddable)]
pub struct Bar {
    pub x: i32,
    pub y: i32,
}

#[flatt(embeddable)]
impl Bar {
    pub fn some_method(&self) -> i32 {
        self.x * self.y
    }
}

#[flatt]
pub struct Foo {
    #[flatt]
    pub bar: Bar,
    pub z: i32,
}
