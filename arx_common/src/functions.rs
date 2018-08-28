use prelude::*;
use std::collections::HashMap;


use std::sync::Mutex;
use std::mem::transmute;

// not a good enough way to use global state for this to work at the moment, though we could presumably just wrap
// everything in a lazy static, it doesn't really seem useful


//lazy_static! {
//    static ref NAMED_FUNCTIONS_1: Mutex<HashMap<Str, fn()>> = Mutex::new(HashMap::new());
//}
//
//struct NamedFunction0<R> {
//    name : Str,
//    function : fn() -> R
//}
//struct NamedFunction1<T, R> {
//    name : Str,
//    function : fn(T) -> R
//}
//
//fn name_function<T, R>(name : Str, func : fn(T) -> R) -> NamedFunction1<T,R> {
//    NAMED_FUNCTIONS_1.lock().unwrap().insert(name, unsafe { transmute::<fn(T) -> R, fn()>(func) });
//    NamedFunction1 { name, function : func }
//}
//
//
//
//#[cfg(test)]
//mod test {
//    use super::*;
//
//    static named_foo : NamedFunction1<i32, i32> = name_function("foo", |a: i32| { a + 1 });
//
//    #[test]
//    pub fn test_named_function () {
//
//    }
//}