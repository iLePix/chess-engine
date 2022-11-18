
#[macro_export]
macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
        let cap = count!($($key)*);
        let mut map = ::std::collections::HashMap::with_capacity(cap);
        $( map.insert($key, $val); )*
        map
    }}
}

#[macro_export]
macro_rules! count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + count!($($xs)*));
}

