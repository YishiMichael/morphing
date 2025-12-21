pub trait Get<'c, K, T> {
    fn get(&'c self, key: K) -> T;
}

impl<'c, K, T, C> Get<'c, K, T> for C
where
    K: 'c,
    C: __private::Select<'c, K>,
    T: __private::Convert<C::Representation>,
{
    fn get(&'c self, key: K) -> T {
        T::convert(self.select(key))
    }
}

#[doc(hidden)]
pub mod __private {
    pub const MAX_ARITY: usize = 16;

    pub trait Select<'c, K> {
        type Representation;

        fn select(&'c self, key: K) -> &'c Self::Representation;
    }

    pub trait Convert<R> {
        fn convert(representation: &R) -> Self;
    }

    pub mod key_types {
        #[derive(Default)]
        pub struct KeySegmentName<const H: u64>;

        #[derive(Default)]
        pub struct KeySegmentIndex<const I: usize>;

        #[derive(Default)]
        pub struct Cons(R, P);
    }

    pub mod representation_types {
        pub struct Nil;

        pub struct Bool(pub bool);

        pub struct StaticStr(pub &'static str);

        pub enum Number {
            Integer(i64),
            Float(f64),
        }

        // use super::select::__representation_types::{Bool, Nil, Number, StaticStr};

        // impl<'c, R, P, C> Select<'c, Cons<R, P>> for C
        // where
        //     C: Select<'c, R>,
        //     C::Representation: Select<'c, P>,
        // {
        //     type Representation = <C::Representation as Select<'c, P>>::Representation;

        //     fn select(&'c self, key: Cons<R, P>) -> &'c Self::Representation {
        //         self.select(key.root).select(key.postfix)
        //     }
        // }

        impl<T> Convert<Nil> for Option<T> {
            fn convert(_representation: &Nil) -> Self {
                None
            }
        }

        impl Convert<Bool> for bool {
            fn convert(representation: &Bool) -> Self {
                representation.0
            }
        }

        impl Convert<StaticStr> for &str {
            fn convert(representation: &StaticStr) -> Self {
                representation.0
            }
        }

        impl Convert<StaticStr> for String {
            fn convert(representation: &StaticStr) -> Self {
                representation.0.to_string()
            }
        }

        macro_rules! numeric_convert {
            ($($target:ty),*) => {
                $(
                    impl Convert<Number> for $target {
                        fn convert(representation: &Number) -> Self {
                            match representation {
                                Number::Integer(value) => *value as Self,
                                Number::Float(value) => *value as Self,
                            }
                        }
                    }
                )*
            };
        }
        numeric_convert!(i32, i64, i128, isize, u32, u64, u128, usize, f32, f64);

        // include!(concat!(env!("OUT_DIR"), "/representation_tuples_impl.rs"));

        // impl Convert<RepresentationEmptyArray> for () {
        //     fn convert(_representation: &RepresentationEmptyArray) -> Self {
        //         ()
        //     }
        // }

        // impl<T> Convert<RepresentationEmptyArray> for [T; 0] {
        //     fn convert(_representation: &RepresentationEmptyArray) -> Self {
        //         []
        //     }
        // }

        // impl<T> Convert<RepresentationEmptyArray> for Vec<T> {
        //     fn convert(_representation: &RepresentationEmptyArray) -> Self {
        //         Vec::new()
        //     }
        // }

        // impl<R, T, const N: usize> Convert<RepresentationArray<R, N>> for [T; N]
        // where
        //     T: Convert<R>,
        // {
        //     fn convert(representation: &RepresentationArray<R, N>) -> Self {
        //         std::array::from_fn(|i| T::convert(&representation.0[i]))
        //     }
        // }

        // impl<R, T, const N: usize> Convert<RepresentationArray<R, N>> for Vec<T>
        // where
        //     T: Convert<R>,
        // {
        //     fn convert(representation: &RepresentationArray<R, N>) -> Self {
        //         <[T; N]>::convert(representation).into()
        //     }
        // }
    }
}

// #[doc(hidden)]
// pub mod __representation_types {
//     pub struct Nil;

//     pub struct Bool(pub bool);

//     pub struct StaticStr(pub &'static str);

//     pub enum Number {
//         Integer(i64),
//         Float(f64),
//     }
// }

// #[doc(hidden)]
// pub const MAX_TUPLE_ARITY: usize = 16;
