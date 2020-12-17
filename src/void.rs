pub enum Void {}

impl<T> FnOnce<T> for Void {
    type Output = ();

    extern "rust-call" fn call_once(self, _args: T) {
        match self {}
    }
}

impl<T> FnMut<T> for Void {
    extern "rust-call" fn call_mut(&mut self, _args: T) {
        match *self {}
    }
}

impl<T> Fn<T> for Void {
    extern "rust-call" fn call(&self, _args: T) {
        match *self {}
    }
}
