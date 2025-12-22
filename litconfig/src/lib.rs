pub use litconfig_macros::{config, key, ConfigData, Key};

pub trait Get<'c, K, T> {
    fn get(&'c self, key: K) -> T;
}

#[doc(hidden)]
pub mod __private {
    // pub const MAX_ARITY: usize = 16;

    pub trait Select<'c, K> {
        type Representation;

        fn select(&'c self, key: K) -> &'c Self::Representation;
    }

    pub trait Convert<'r, T> {
        fn convert(&'r self) -> T;
    }

    pub trait ConfigData<'r, R> {
        fn from(representation: &'r R) -> Self;
    }

    mod representation_impls {
        use super::super::Get;
        use super::{ConfigData, Convert, Select};

        impl<'c, C, K, T> Get<'c, K, T> for C
        where
            K: 'c,
            C: Select<'c, K>,
            C::Representation: Convert<'c, T>,
        {
            fn get(&'c self, key: K) -> T {
                <C::Representation as Convert<'c, T>>::convert(<C as Select<'c, K>>::select(
                    self, key,
                ))
            }
        }

        // pub struct Nil;

        // pub struct Bool(pub bool);

        // pub struct StaticStr(pub &'static str);

        // pub enum Number {
        //     Integer(i64),
        //     Float(f64),
        // }

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

        // impl<T> Convert<'_, T> for ()
        // // TODO
        // where
        //     T: Default,
        // {
        //     fn convert(&self) -> T {
        //         T::default()
        //     }
        // }

        impl Convert<'_, bool> for bool {
            fn convert(&self) -> bool {
                *self
            }
        }

        impl<'r> Convert<'r, &'r str> for &'static str {
            fn convert(&'r self) -> &'r str {
                *self
            }
        }

        impl Convert<'_, String> for &'static str {
            fn convert(&self) -> String {
                self.to_string()
            }
        }

        macro_rules! numeric_convert {
            ($($target:ty),*) => {
                $(
                    impl Convert<'_, $target> for i64 {
                        fn convert(&self) -> $target {
                            *self as $target
                        }
                    }

                    impl Convert<'_, $target> for f64 {
                        fn convert(&self) -> $target {
                            *self as $target
                        }
                    }
                )*
            };
        }
        numeric_convert!(i32, i64, i128, isize, u32, u64, u128, usize, f32, f64);

        impl<'r, R, T> Convert<'r, T> for R
        where
            T: ConfigData<'r, R>,
        {
            fn convert(&'r self) -> T {
                <T as ConfigData<'r, R>>::from(self)
            }
        }

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

    pub mod key_types {
        use super::Select;

        #[derive(Default)]
        pub struct KeySegmentName<const H: u64>;

        #[derive(Default)]
        pub struct KeySegmentIndex<const I: usize>;

        #[derive(Default)]
        pub struct Cons<KS, KSS>(KS, KSS);

        impl<'c, C, KSS, const H: u64> Select<'c, Cons<KeySegmentName<H>, KSS>> for C
        where
            C: Select<'c, KeySegmentName<H>>,
            C::Representation: Select<'c, KSS>,
        {
            type Representation = <C::Representation as Select<'c, KSS>>::Representation;

            fn select(&'c self, key: Cons<KeySegmentName<H>, KSS>) -> &'c Self::Representation {
                <C::Representation as Select<'c, KSS>>::select(
                    <C as Select<'c, KeySegmentName<H>>>::select(self, key.0),
                    key.1,
                )
            }
        }

        impl<'c, C, KSS, const I: usize> Select<'c, Cons<KeySegmentIndex<I>, KSS>> for C
        where
            C: Select<'c, KeySegmentIndex<I>>,
            C::Representation: Select<'c, KSS>,
        {
            type Representation = <C::Representation as Select<'c, KSS>>::Representation;

            fn select(&'c self, key: Cons<KeySegmentIndex<I>, KSS>) -> &'c Self::Representation {
                <C::Representation as Select<'c, KSS>>::select(
                    <C as Select<'c, KeySegmentIndex<I>>>::select(self, key.0),
                    key.1,
                )
            }
        }
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

// struct MyC {
//     a: i64,
//     b: String,
// }

// impl<'c, R> crate::__private::Convert<'c, MyC> for R
// where
//     R: crate::__private::Select<'c, usize>,
//     <R as crate::__private::Select<'c, usize>>::Representation: crate::__private::Convert<'c, i64>,
//     R: crate::__private::Select<'c, ()>,
//     <R as crate::__private::Select<'c, ()>>::Representation: crate::__private::Convert<'c, String>,
// {
//     fn convert(&'c self) -> MyC {
//         MyC {
//             a: self.select(<usize>::default()).convert(),
//             b: self.select(<()>::default()).convert(),
//         }
//     }
// }
