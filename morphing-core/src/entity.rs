use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;

use super::storable::SlotKeyGeneratorTypeMap;
use super::storable::StorageTypeMap;
use super::timer::Clock;
use super::timer::ClockSpan;

pub trait Mobject:
    'static + Clone + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize
{
    // type Children<V, S> where V: Variant<M>, S: Stage;
    // type Entity;
    // type GenericChildren<C, S>;
}

// pub trait Children:
//     'static + Clone + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize
// {
// }

// #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
// pub struct Entity<M, C> {
//     mobject: M,
//     children: C,
// }

pub trait Stage {}

pub trait Variant<M, S>
where
    M: Mobject,
    S: Stage,
{
    type Mobject;
    type Children;
    type Extra;
}

// impl<C> Variant for GenericVariant<C> {
//     type Children<M, S> = Entity<Arc<M>, M::GenericChildren<C, S>> where M: Mobject, S: Stage;
//     // type Children<M> = M::Children where M: Mobject;
// }

pub trait Refresh<M>: Send + Sync
where
    M: Mobject,
{
    fn refresh(
        &self,
        clock: Clock,
        clock_span: ClockSpan,
        entity: Entity<M, M::Children>,
    ) -> Entity<M, M::Children>;
}

trait Allocate {
    type Output;

    fn allocate(self, slot_key_generator_map: &SlotKeyGeneratorTypeMap) -> Self::Output;
}

trait Observe {
    type Output;

    fn observe(
        &self,
        storage_type_map: &StorageTypeMap,
        clock: Clock,
        clock_span: ClockSpan,
    ) -> Self::Output;
}

pub struct TransferStage;
pub struct AllocateStage;
pub struct ObserveStage;

impl Stage for TransferStage {}

// impl Morphism<(&SlotKeyGeneratorTypeMap,)> for TransferStage {
//     type Output = AllocateStage;

//     fn morphism(self, input: (&SlotKeyGeneratorTypeMap,)) -> Self::Output {
//         AllocateStage
//     }
// }

impl Stage for AllocateStage {}

// impl MorphismRef<(&StorageTypeMap, Clock, ClockSpan)> for AllocateStage {
//     type Output = ObserveStage;

//     fn morphism_ref(&self, input: (&StorageTypeMap, Clock, ClockSpan)) -> Self::Output {
//         ObserveStage
//     }
// }

impl Stage for ObserveStage {}

pub struct Entity<M, S, V>
where
    M: Mobject,
    S: Stage,
    V: Variant<M, S>,
{
    mobject: V::Mobject,
    children: V::Children,
    extra: V::Extra,
}

impl<M, S, V> Deref for Entity<M, S, V>
where
    M: Mobject,
    S: Stage,
    V: Variant<M, S>,
{
    type Target = V::Mobject;

    fn deref(&self) -> &Self::Target {
        &self.mobject
    }
}

// impl<I, M, S, V> Morphism<I> for Entity<M, S, V>
// where
//     I: Clone,
//     M: Mobject,
//     V: Variant<M, S> + Variant<M, S::Output>,
//     <V as Variant<M, S>>::Children: Morphism<I, Output = <V as Variant<M, S::Output>>::Children>,
//     <V as Variant<M, S>>::Extra: Morphism<I, Output = <V as Variant<M, S::Output>>::Extra>,
//     S: Stage + Morphism<I>,
//     S::Output: Stage,
// {
//     type Output = Entity<M, S, V::Output>;

//     fn morphism(self, input: I) -> Self::Output {
//         Entity {
//             mobject: self.mobject,
//             children: self.children.morphism(input.clone()),
//             extra: self.extra.morphism(input),
//         }
//     }
// }

// demo code

// atom

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject0Children {}

// impl Children for MyMobject0Children {}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject0 {
    demo0: f32,
}

impl Mobject for MyMobject0 {
    // type Children = MyMobject0Children;
    // type Entity = Entity<MyMobject0, MyMobject0Children>;
}

pub trait MyMobject0Trait {}

impl MyMobject0Trait for MyMobject0Children {}

impl<S, V> MyMobject0Trait for Entity<MyMobject0, S, V>
where
    S: Stage,
    V: Variant<MyMobject0, S>,
    V::Children: MyMobject0Trait,
{
}

pub struct MyMobject0StaticVariant;

impl Variant<MyMobject0, TransferStage> for MyMobject0StaticVariant {
    type Mobject = Arc<MyMobject0>;
    type Children = MyMobject0Children;
    type Extra = ();
}

impl Variant<MyMobject0, AllocateStage> for MyMobject0StaticVariant {
    type Mobject = Arc<MyMobject0>;
    type Children = MyMobject0Children;
    type Extra = ();
}

impl Variant<MyMobject0, ObserveStage> for MyMobject0StaticVariant {
    type Mobject = Arc<MyMobject0>;
    type Children = MyMobject0Children;
    type Extra = ();
}

impl Allocate for Entity<MyMobject0, TransferStage, MyMobject0StaticVariant> {
    type Output = Entity<MyMobject0, AllocateStage, MyMobject0StaticVariant>;

    fn allocate(self, slot_key_generator_map: &SlotKeyGeneratorTypeMap) -> Self::Output {
        Entity {
            mobject: self.mobject,
            children: self.children.allocate(slot_key_generator_map),
            extra: self.extra,
        }
    }
}

impl Observe for Entity<MyMobject0, AllocateStage, MyMobject0StaticVariant> {
    type Output = Entity<MyMobject0, ObserveStage, MyMobject0StaticVariant>;

    fn observe(
        &self,
        storage_type_map: &StorageTypeMap,
        clock: Clock,
        clock_span: ClockSpan,
    ) -> Self::Output {
        Entity {
            mobject: self.mobject.clone(), //?
            children: self.children.observe(storage_type_map, clock, clock_span),
            extra: self.extra,
        }
    }
}

pub struct MyMobject0DynamicVariant<R>(R);

impl<R> Variant<MyMobject0, TransferStage> for MyMobject0DynamicVariant<R>
where
    R: Refresh<MyMobject0>,
{
    type Mobject = MyMobject0;
    type Children = MyMobject0Children;
    type Extra = (R,);
}

impl<R> Variant<MyMobject0, AllocateStage> for MyMobject0DynamicVariant<R>
where
    R: Refresh<MyMobject0>,
{
    type Mobject = MyMobject0;
    type Children = MyMobject0Children;
    type Extra = (R,);
}

impl<R> Variant<MyMobject0, ObserveStage> for MyMobject0DynamicVariant<R>
where
    R: Refresh<MyMobject0>,
{
    type Mobject = MyMobject0;
    type Children = MyMobject0Children;
    type Extra = ();
}

// pub struct MyMobject0GenericVariant();

impl Variant<MyMobject0, TransferStage> for MyMobject0Children {
    type Mobject = Arc<MyMobject0>;
    type Children = MyMobject0Children;
    type Extra = ();
}

impl Variant<MyMobject0, AllocateStage> for MyMobject0Children {
    type Mobject = Arc<MyMobject0>;
    type Children = MyMobject0Children;
    type Extra = ();
}

impl Variant<MyMobject0, ObserveStage> for MyMobject0Children {
    type Mobject = Arc<MyMobject0>;
    type Children = MyMobject0Children;
    type Extra = ();
}

impl Allocate for MyMobject0Children {
    type Output = MyMobject0Children;

    #[allow(unused_variables)]
    fn allocate(self, slot_key_generator_map: &SlotKeyGeneratorTypeMap) -> Self::Output {
        MyMobject0Children {}
    }
}

impl Observe for MyMobject0Children {
    type Output = MyMobject0Children;

    #[allow(unused_variables)]
    fn observe(
        &self,
        storage_type_map: &StorageTypeMap,
        clock: Clock,
        clock_span: ClockSpan,
    ) -> Self::Output {
        MyMobject0Children {}
    }
}

// atom structured

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject1Children<
    MA,
    MB,
    // MA = Entity<MyMobject0, MyMobject0Children>,
    // MB = Entity<MyMobject0, MyMobject0Children>,
> {
    ma: MA,
    mb: MB,
}

// impl Children for MyMobject1Children {}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject1 {
    demo1: f32,
}

impl Mobject for MyMobject1 {
    // type Children = MyMobject1Children;
    // type Entity = Entity<MyMobject1, MyMobject1Children>;
}

pub trait MyMobject1Trait {
    type MA;
    type MB;

    fn ma(&self) -> &Self::MA;
    fn mb(&self) -> &Self::MB;
}

impl<MA, MB> MyMobject1Trait for MyMobject1Children<MA, MB> {
    type MA = MA;
    type MB = MB;

    fn ma(&self) -> &Self::MA {
        &self.ma
    }

    fn mb(&self) -> &Self::MB {
        &self.mb
    }
}

impl<S, V> MyMobject1Trait for Entity<MyMobject1, S, V>
where
    S: Stage,
    V: Variant<MyMobject1, S>,
    V::Children: MyMobject1Trait,
{
    type MA = <V::Children as MyMobject1Trait>::MA;
    type MB = <V::Children as MyMobject1Trait>::MB;

    fn ma(&self) -> &Self::MA {
        self.children.ma()
    }

    fn mb(&self) -> &Self::MB {
        self.children.mb()
    }
}

pub struct MyMobject1StaticVariant;

impl Variant<MyMobject1, TransferStage> for MyMobject1StaticVariant {
    type Children = MyMobject1Children<
        Entity<MyMobject0, TransferStage, MyMobject0StaticVariant>,
        Entity<MyMobject0, TransferStage, MyMobject0StaticVariant>,
    >;
    type Extra = ();
}

impl Variant<MyMobject1, AllocateStage> for MyMobject1StaticVariant {
    type Children = MyMobject1Children<
        Entity<MyMobject0, AllocateStage, MyMobject0StaticVariant>,
        Entity<MyMobject0, AllocateStage, MyMobject0StaticVariant>,
    >;
    type Extra = ();
}

impl Variant<MyMobject1, ObserveStage> for MyMobject1StaticVariant {
    type Children = MyMobject1Children<
        Entity<MyMobject0, ObserveStage, MyMobject0StaticVariant>,
        Entity<MyMobject0, ObserveStage, MyMobject0StaticVariant>,
    >;
    type Extra = ();
}

pub struct MyMobject1DynamicVariant<R>(R);

impl<R> Variant<MyMobject1, TransferStage> for MyMobject1DynamicVariant<R>
where
    R: Refresh<MyMobject1>,
{
    type Children = MyMobject1Children<
        Entity<MyMobject0, TransferStage, MyMobject0StaticVariant>,
        Entity<MyMobject0, TransferStage, MyMobject0StaticVariant>,
    >;
    type Extra = (R,);
}

impl<R> Variant<MyMobject1, AllocateStage> for MyMobject1DynamicVariant<R>
where
    R: Refresh<MyMobject1>,
{
    type Children = MyMobject1Children<
        Entity<MyMobject0, AllocateStage, MyMobject0StaticVariant>,
        Entity<MyMobject0, AllocateStage, MyMobject0StaticVariant>,
    >;
    type Extra = (R,);
}

impl<R> Variant<MyMobject1, ObserveStage> for MyMobject1DynamicVariant<R>
where
    R: Refresh<MyMobject1>,
{
    type Children = MyMobject1Children<
        Entity<MyMobject0, ObserveStage, MyMobject0StaticVariant>,
        Entity<MyMobject0, ObserveStage, MyMobject0StaticVariant>,
    >;
    type Extra = ();
}

// pub struct MyMobject1GenericVariant<MA, MB> {
//     ma: MA,
//     mb: MB,
// }

impl<MA, MB> Variant<MyMobject1, TransferStage> for MyMobject1Children<MA, MB>
where
    MA: Variant<MyMobject0, TransferStage>,
    MB: Variant<MyMobject0, TransferStage>,
{
    type Children = MyMobject1Children<
        Entity<MyMobject0, TransferStage, MA>,
        Entity<MyMobject0, TransferStage, MB>,
    >;
    type Extra = ();
}

impl<MA, MB> Variant<MyMobject1, AllocateStage> for MyMobject1Children<MA, MB>
where
    MA: Variant<MyMobject0, AllocateStage>,
    MB: Variant<MyMobject0, AllocateStage>,
{
    type Children = MyMobject1Children<
        Entity<MyMobject0, AllocateStage, MA>,
        Entity<MyMobject0, AllocateStage, MB>,
    >;
    type Extra = ();
}

impl<MA, MB> Variant<MyMobject1, ObserveStage> for MyMobject1Children<MA, MB>
where
    MA: Variant<MyMobject0, ObserveStage>,
    MB: Variant<MyMobject0, ObserveStage>,
{
    type Children = MyMobject1Children<
        Entity<MyMobject0, ObserveStage, MA>,
        Entity<MyMobject0, ObserveStage, MB>,
    >;
    type Extra = ();
}

impl<MA, MB> Allocate for MyMobject1Children<MA, MB>
where
    MA: Allocate,
    MB: Allocate,
{
    type Output = MyMobject1Children<MA::Output, MB::Output>;

    #[allow(unused_variables)]
    fn allocate(self, slot_key_generator_map: &SlotKeyGeneratorTypeMap) -> Self::Output {
        MyMobject1Children {
            ma: self.ma.allocate(slot_key_generator_map),
            mb: self.mb.allocate(slot_key_generator_map),
        }
    }
}

impl<MA, MB> Observe for MyMobject1Children<MA, MB>
where
    MA: Observe,
    MB: Observe,
{
    type Output = MyMobject1Children<MA::Output, MB::Output>;

    #[allow(unused_variables)]
    fn observe(
        &self,
        storage_type_map: &StorageTypeMap,
        clock: Clock,
        clock_span: ClockSpan,
    ) -> Self::Output {
        MyMobject1Children {
            ma: self.ma.observe(storage_type_map, clock, clock_span),
            mb: self.mb.observe(storage_type_map, clock, clock_span),
        }
    }
}
