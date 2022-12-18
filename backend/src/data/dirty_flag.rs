pub struct DirtyFlag<T> {
    is_dirty: bool,
    data: T,
}

impl<T> DirtyFlag<T> {
    pub fn new(data: T) -> Self {
        Self { is_dirty: false, data }
    }

    pub fn read(&self) -> &T {
        if self.is_dirty {
            panic!("Read dirtied data!");
        }

        &self.data
    }

    pub fn unchecked_read(&self) -> &T {
        &self.data
    }

    pub fn write(&mut self) -> &mut T {
        self.is_dirty = true;
        &mut self.data
    }

    pub fn get_all(&mut self) -> (&mut T, &mut bool) {
        (&mut self.data, &mut self.is_dirty)
    }

    pub fn is_dirty(&mut self) -> bool {
        self.is_dirty
    }

    pub fn clean_flag(&mut self) {
        self.is_dirty = false;
    }

    pub fn set_dirty(&mut self) {
        self.is_dirty = true;
    }
}

#[cfg(test)]
mod test_data {
    use super::*;

    #[test]
    fn test_dirty_flag() {
        let mut num = DirtyFlag::new(3);

        assert!(!num.is_dirty());
        assert_eq!(*num.read(), 3);
        assert!(!num.is_dirty());

        *num.write() = 2;
        assert_eq!(*num.unchecked_read(), 2);
        assert!(num.is_dirty());
        num.clean_flag();
        assert!(!num.is_dirty());
        num.set_dirty();
        assert!(num.is_dirty());
    }

    #[test]
    #[should_panic(expected = "Read dirtied data!")]
    fn test_dirty_flag_panic() {
        let mut ok = DirtyFlag::new(true);
        *ok.write() = false;
        ok.read();
    }
}