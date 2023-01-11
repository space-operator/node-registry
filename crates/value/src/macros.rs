#[macro_export]
macro_rules! map {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$($crate::map!(@single $rest)),*]));

    ($($key:expr => $value:expr,)+) => { $crate::map!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let _cap = $crate::map!(@count $($key),*);
            let mut _map = $crate::Map::with_capacity(_cap);
            $(
                let _ = _map.insert($crate::Key::from($key), $crate::Value::from($value));
            )*
            _map
        }
    };
}
