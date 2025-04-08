use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;

use super::storable::ArcSlot;
use super::storable::ResourceReuseResult;
use super::storable::SlotKeyGeneratorTypeMap;
use super::storable::StorageKey;
use super::storable::StorageTypeMap;
use super::storable::StoreType;
use super::storable::VecSlot;
use super::timer::Clock;
use super::timer::ClockSpan;

pub trait Mobject:
    'static + Clone + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize
{
    type StaticKey;
    type DynamicKey;
    type Resource: 'static + Send + Sync;
    type ResourceRefInput<'s>;
    type ResourceRef<'s>: Send + Sync;

    fn static_allocate(
        mobject: &Self,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::StaticKey;
    fn dynamic_allocate(
        mobject: &Self,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::DynamicKey;
    fn static_prepare<'s>(
        mobject: &Self,
        key: &Self::StaticKey,
        storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        result: &mut ResourceReuseResult,
    ) -> Self::ResourceRef<'s>;
    fn dynamic_prepare<'s>(
        mobject: &Self,
        key: &Self::DynamicKey,
        storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        result: &mut ResourceReuseResult,
    ) -> Self::ResourceRef<'s>;
    // fn resource_input<'m>(mobject: &'m Self) -> Self::ResourceInput<'s>;
    // fn prepare_new(
    //     mobject: &Self,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    // ) -> Self::Resource;
    // fn prepare_incremental(
    //     resource: &mut Self::Resource,
    //     mobject: &Self,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    // ) -> ResourceReuseResult;
    fn render(resource: Self::ResourceRef<'_>, render_pass: &mut wgpu::RenderPass);
}

pub trait Timeline<M>: 'static + Send + Sized + Sync
where
    M: Mobject,
{
    type Observe: Send + Sync;
    type Key;

    fn allocate(
        timeline: &Self,
        observe: &Self::Observe,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Key;
    fn prepare(
        clock: Clock,
        clock_span: ClockSpan,
        timeline: &Self,
        observe: &Self::Observe,
        key: &Self::Key,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> ResourceReuseResult;
}

pub trait Resource<M>
where
    M: Mobject,
{
    fn prepare_new(
        resource_input: M::ResourceRefInput<'_>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self;
    fn prepare_incremental(
        &mut self,
        resource_input: M::ResourceRefInput<'_>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> ResourceReuseResult;
    fn render(&self, render_pass: &mut wgpu::RenderPass);
}

pub trait Variant<M>: StoreType
where
    M: Mobject,
{
}

pub struct StaticVariant<M>(M);

impl<M> StoreType for StaticVariant<M>
where
    M: Mobject,
{
    type KeyInput = M;
    type Slot = ArcSlot<M::Resource>;
}

impl<M> Variant<M> for StaticVariant<M> where M: Mobject {}

pub struct DynamicVariant<M>(M);

impl<M> StoreType for DynamicVariant<M>
where
    M: Mobject,
{
    type KeyInput = ();
    type Slot = VecSlot<M::Resource>;
}

impl<M> Variant<M> for DynamicVariant<M> where M: Mobject {}

pub trait Refresh<M>: 'static + Send + Sync
where
    M: Mobject,
{
    fn refresh(&self, clock: Clock, clock_span: ClockSpan, mobject: M) -> M;
}

pub struct StaticTimeline;

// impl<M> Storable for (M, StaticTimeline)
// where
//     M: Mobject,
// {
//     type KeyInput = M;
//     type Slot = ArcSlot<M::Resource>;
// }

impl<M> Timeline<M> for StaticTimeline
where
    M: Mobject,
{
    type Observe = Arc<M>;
    type Key = M::StaticKey;

    fn allocate(
        _timeline: &Self,
        observe: &Self::Observe,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Key {
        M::static_allocate(observe.as_ref(), slot_key_generator_map)
        // slot_key_generator_map.allocate(observe.as_ref())
    }

    fn prepare(
        _clock: Clock,
        _clock_span: ClockSpan,
        _timeline: &Self,
        observe: &Self::Observe,
        key: &Self::Key,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> ResourceReuseResult {
        let mut result = Ok(());
        M::static_prepare(
            observe.as_ref(),
            key,
            storage_type_map,
            device,
            queue,
            format,
            &mut result,
        );
        result
        // storage_type_map
        //     .update_or_insert(
        //         key,
        //         || {
        //             Arc::new(M::prepare_new(
        //                 &M::resource_input(observe.as_ref()),
        //                 device,
        //                 queue,
        //                 format,
        //             ))
        //         },
        //         |_| Ok(()),
        //     )
        //     .unwrap_or(Err(()))
    }
}

pub struct DynamicTimeline<R> {
    refresh: R,
}

impl<M, R> Timeline<M> for DynamicTimeline<R>
where
    M: Mobject,
    R: Refresh<M>,
{
    type Observe = M;
    type Key = M::DynamicKey;

    fn allocate(
        _timeline: &Self,
        observe: &Self::Observe,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Key {
        M::dynamic_allocate(observe, slot_key_generator_map)
        // slot_key_generator_map.allocate(&())
    }

    fn prepare(
        clock: Clock,
        clock_span: ClockSpan,
        timeline: &Self,
        observe: &Self::Observe,
        key: &Self::Key,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> ResourceReuseResult {
        let mut result = Ok(());
        M::dynamic_prepare(
            &timeline.refresh.refresh(clock, clock_span, observe.clone()),
            key,
            storage_type_map,
            device,
            queue,
            format,
            &mut result,
        );
        result
        // let resource_input = M::resource_input(&mobject);
        // storage_type_map
        //     .update_or_insert(
        //         key,
        //         || M::prepare_new(&resource_input, device, queue, format),
        //         |resource| M::prepare_incremental(resource, &resource_input, device, queue, format),
        //     )
        //     .unwrap_or(Err(()))
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Data<T>(T);

impl<T> From<T> for Data<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> Deref for Data<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Mobject for Data<T>
where
    T: 'static + Clone + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize,
{
    type StaticKey = ();
    type DynamicKey = ();
    type Resource = ();
    type ResourceRefInput<'s> = ();
    type ResourceRef<'s> = T;

    fn static_allocate(
        _mobject: &Self,
        _slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::StaticKey {
        ()
    }

    fn dynamic_allocate(
        _mobject: &Self,
        _slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::DynamicKey {
        ()
    }

    fn static_prepare<'s>(
        mobject: &Self,
        _key: &Self::StaticKey,
        _storage_type_map: &'s mut StorageTypeMap,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
        _result: &mut ResourceReuseResult,
    ) -> Self::ResourceRef<'s> {
        (**mobject).clone()
    }

    fn dynamic_prepare<'s>(
        mobject: &Self,
        _key: &Self::DynamicKey,
        _storage_type_map: &'s mut StorageTypeMap,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
        _result: &mut ResourceReuseResult,
    ) -> Self::ResourceRef<'s> {
        (**mobject).clone()
    }

    // fn resource_input<'m>(mobject: &'m Self) -> Self::ResourceInput<'s> {
    //     mobject
    // }

    // fn prepare_new(
    //     mobject: &Self,
    //     _device: &wgpu::Device,
    //     _queue: &wgpu::Queue,
    //     _format: wgpu::TextureFormat,
    // ) -> Self::Resource {
    //     (**mobject).clone()
    // }

    // fn prepare_incremental(
    //     _resource: &mut Self::Resource,
    //     _mobject: &Self,
    //     _device: &wgpu::Device,
    //     _queue: &wgpu::Queue,
    //     _format: wgpu::TextureFormat,
    // ) -> ResourceReuseResult {
    //     Ok(())
    // }

    fn render(_resource: Self::ResourceRef<'_>, _render_pass: &mut wgpu::RenderPass) {}
}

// demo

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject0<MA = Data<f32>, MB = Data<f32>> {
    ma: MA,
    mb: MB,
}

impl Mobject for MyMobject0 {
    type StaticKey =
        MyMobject0<<Data<f32> as Mobject>::StaticKey, <Data<f32> as Mobject>::StaticKey>;
    type DynamicKey =
        MyMobject0<<Data<f32> as Mobject>::DynamicKey, <Data<f32> as Mobject>::DynamicKey>;
    type Resource = ();
    type ResourceRefInput<'s> = MyMobject0<
        <Data<f32> as Mobject>::ResourceRef<'s>,
        <Data<f32> as Mobject>::ResourceRef<'s>,
    >;
    type ResourceRef<'s> = MyMobject0<
        <Data<f32> as Mobject>::ResourceRef<'s>,
        <Data<f32> as Mobject>::ResourceRef<'s>,
    >;

    fn static_allocate(
        mobject: &Self,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::StaticKey {
        MyMobject0 {
            ma: <Data<f32> as Mobject>::static_allocate(&mobject.ma, slot_key_generator_map),
            mb: <Data<f32> as Mobject>::static_allocate(&mobject.mb, slot_key_generator_map),
        }
    }

    fn dynamic_allocate(
        mobject: &Self,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::DynamicKey {
        MyMobject0 {
            ma: <Data<f32> as Mobject>::dynamic_allocate(&mobject.ma, slot_key_generator_map),
            mb: <Data<f32> as Mobject>::dynamic_allocate(&mobject.mb, slot_key_generator_map),
        }
    }

    fn static_prepare<'s>(
        mobject: &Self,
        key: &Self::StaticKey,
        storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        result: &mut ResourceReuseResult,
    ) -> Self::ResourceRef<'s> {
        MyMobject0 {
            ma: <Data<f32> as Mobject>::static_prepare(
                &mobject.ma,
                &key.ma,
                storage_type_map,
                device,
                queue,
                format,
                result,
            ),
            mb: <Data<f32> as Mobject>::static_prepare(
                &mobject.mb,
                &key.mb,
                storage_type_map,
                device,
                queue,
                format,
                result,
            ),
        }
    }

    fn dynamic_prepare<'s>(
        mobject: &Self,
        key: &Self::DynamicKey,
        storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        result: &mut ResourceReuseResult,
    ) -> Self::ResourceRef<'s> {
        MyMobject0 {
            ma: <Data<f32> as Mobject>::dynamic_prepare(
                &mobject.ma,
                &key.ma,
                storage_type_map,
                device,
                queue,
                format,
                result,
            ),
            mb: <Data<f32> as Mobject>::dynamic_prepare(
                &mobject.mb,
                &key.mb,
                storage_type_map,
                device,
                queue,
                format,
                result,
            ),
        }
    }

    // fn resource_input<'m>(mobject: &'m Self) -> Self::ResourceInput<'s> {
    //     MyMobject0 {
    //         ma: <Data<f32> as Mobject>::resource_input(&mobject.ma),
    //         mb: <Data<f32> as Mobject>::resource_input(&mobject.mb),
    //     }
    // }

    // fn prepare_new(
    //     mobject: &Self,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    // ) -> Self::Resource {
    //     MyMobject0 {
    //         ma: <Data<f32> as Mobject>::prepare_new(&mobject.ma, device, queue, format),
    //         mb: <Data<f32> as Mobject>::prepare_new(&mobject.mb, device, queue, format),
    //     }
    // }

    // fn prepare_incremental(
    //     resource: &mut Self::Resource,
    //     mobject: &Self,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    // ) -> ResourceReuseResult {
    //     [
    //         <Data<f32> as Mobject>::prepare_incremental(
    //             &mut resource.ma,
    //             &mobject.ma,
    //             device,
    //             queue,
    //             format,
    //         ),
    //         <Data<f32> as Mobject>::prepare_incremental(
    //             &mut resource.mb,
    //             &mobject.mb,
    //             device,
    //             queue,
    //             format,
    //         ),
    //     ]
    //     .into_iter()
    //     .collect()
    // }

    fn render(resource: Self::ResourceRef<'_>, render_pass: &mut wgpu::RenderPass) {
        <Data<f32> as Mobject>::render(resource.ma, render_pass);
        <Data<f32> as Mobject>::render(resource.mb, render_pass);
    }
}

impl<MA, MB> Timeline<MyMobject0<Data<f32>, Data<f32>>> for MyMobject0<MA, MB>
where
    MA: Timeline<Data<f32>>,
    MB: Timeline<Data<f32>>,
{
    type Observe = MyMobject0<MA::Observe, MB::Observe>;
    type Key = MyMobject0<MA::Key, MB::Key>;

    fn allocate(
        timeline: &Self,
        observe: &Self::Observe,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Key {
        MyMobject0 {
            ma: MA::allocate(&timeline.ma, &observe.ma, slot_key_generator_map),
            mb: MB::allocate(&timeline.mb, &observe.mb, slot_key_generator_map),
        }
    }

    fn prepare(
        clock: Clock,
        clock_span: ClockSpan,
        timeline: &Self,
        observe: &Self::Observe,
        key: &Self::Key,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> ResourceReuseResult {
        [
            MA::prepare(
                clock,
                clock_span,
                &timeline.ma,
                &observe.ma,
                &key.ma,
                storage_type_map,
                device,
                queue,
                format,
            ),
            MB::prepare(
                clock,
                clock_span,
                &timeline.mb,
                &observe.mb,
                &key.mb,
                storage_type_map,
                device,
                queue,
                format,
            ),
        ]
        .into_iter()
        .collect()
    }
}

//

pub struct MyMobject1Resource([f32; 4]);

impl Resource<MyMobject1> for MyMobject1Resource {
    fn prepare_new(
        resource_input: <MyMobject1 as Mobject>::ResourceRefInput<'_>,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
    ) -> Self {
        Self([
            resource_input.ma.ma,
            resource_input.ma.mb,
            resource_input.mb.ma,
            resource_input.mb.mb,
        ])
    }

    fn prepare_incremental(
        &mut self,
        resource_input: <MyMobject1 as Mobject>::ResourceRefInput<'_>,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
    ) -> ResourceReuseResult {
        self.0 = [
            resource_input.ma.ma,
            resource_input.ma.mb,
            resource_input.mb.ma,
            resource_input.mb.mb,
        ];
        Err(())
    }

    fn render(&self, _render_pass: &mut wgpu::RenderPass) {}
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject1<MA = MyMobject0, MB = MyMobject0> {
    ma: MA,
    mb: MB,
}

impl Mobject for MyMobject1 {
    type StaticKey = (
        MyMobject1<<MyMobject0 as Mobject>::StaticKey, <MyMobject0 as Mobject>::StaticKey>,
        StorageKey<StaticVariant<MyMobject1>>,
    );
    type DynamicKey = (
        MyMobject1<<MyMobject0 as Mobject>::StaticKey, <MyMobject0 as Mobject>::StaticKey>,
        StorageKey<DynamicVariant<MyMobject1>>,
    );
    type Resource = MyMobject1Resource;
    type ResourceRefInput<'s> = MyMobject1<
        <MyMobject0 as Mobject>::ResourceRef<'s>,
        <MyMobject0 as Mobject>::ResourceRef<'s>,
    >;
    type ResourceRef<'s> = &'s MyMobject1Resource;

    fn static_allocate(
        mobject: &Self,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::StaticKey {
        (
            MyMobject1 {
                ma: <MyMobject0 as Mobject>::static_allocate(&mobject.ma, slot_key_generator_map),
                mb: <MyMobject0 as Mobject>::static_allocate(&mobject.mb, slot_key_generator_map),
            },
            slot_key_generator_map.allocate(mobject),
        )
    }

    fn dynamic_allocate(
        mobject: &Self,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::DynamicKey {
        (
            MyMobject1 {
                ma: <MyMobject0 as Mobject>::dynamic_allocate(&mobject.ma, slot_key_generator_map),
                mb: <MyMobject0 as Mobject>::dynamic_allocate(&mobject.mb, slot_key_generator_map),
            },
            slot_key_generator_map.allocate(&()),
        )
    }

    fn static_prepare<'s>(
        mobject: &Self,
        key: &Self::StaticKey,
        storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        result: &mut ResourceReuseResult,
    ) -> Self::ResourceRef<'s> {
        let resource_input = MyMobject1 {
            ma: <MyMobject0 as Mobject>::static_prepare(
                &mobject.ma,
                &key.0.ma,
                storage_type_map,
                device,
                queue,
                format,
                result,
            ),
            mb: <MyMobject0 as Mobject>::static_prepare(
                &mobject.mb,
                &key.0.mb,
                storage_type_map,
                device,
                queue,
                format,
                result,
            ),
        };
        storage_type_map.update_or_insert(
            &key.1,
            (resource_input, device, queue, format),
            result,
            |(resource_input, device, queue, format)| {
                Arc::new(<MyMobject1Resource as Resource<MyMobject1>>::prepare_new(
                    resource_input,
                    device,
                    queue,
                    format,
                ))
            },
            |(_resource_input, _device, _queue, _format), _resource, _result| {},
        )

        // [
        //     <MyMobject0 as Mobject>::static_prepare(
        //         &mobject.ma,
        //         &key.ma,
        //         storage_type_map,
        //         device,
        //         queue,
        //         format,
        //         result,
        //     ),
        //     <MyMobject0 as Mobject>::static_prepare(
        //         &mobject.mb,
        //         &key.mb,
        //         storage_type_map,
        //         device,
        //         queue,
        //         format,
        //         result,
        //     ),
        // ]
        // .into_iter()
        // .collect()
    }

    fn dynamic_prepare<'s>(
        mobject: &Self,
        key: &Self::DynamicKey,
        storage_type_map: &'s mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        result: &mut ResourceReuseResult,
    ) -> Self::ResourceRef<'s> {
        let resource_input = MyMobject1 {
            ma: <MyMobject0 as Mobject>::dynamic_prepare(
                &mobject.ma,
                &key.0.ma,
                storage_type_map,
                device,
                queue,
                format,
                result,
            ),
            mb: <MyMobject0 as Mobject>::dynamic_prepare(
                &mobject.mb,
                &key.0.mb,
                storage_type_map,
                device,
                queue,
                format,
                result,
            ),
        };
        storage_type_map.update_or_insert(
            &key.1,
            (resource_input, device, queue, format),
            result,
            |(resource_input, device, queue, format)| {
                <MyMobject1Resource as Resource<MyMobject1>>::prepare_new(
                    resource_input,
                    device,
                    queue,
                    format,
                )
            },
            |(resource_input, device, queue, format), resource, result| {
                *result = <MyMobject1Resource as Resource<MyMobject1>>::prepare_incremental(
                    resource,
                    resource_input,
                    device,
                    queue,
                    format,
                );
            },
        )
        // storage_type_map.update_or_insert(
        //     &key.1,
        //     (resource_input, device, queue, format),
        //     result,
        //     |(resource_input, device, queue, format)| {
        //         Arc::new(<MyMobject1Resource as Resource<MyMobject1>>::prepare_new(
        //             resource_input,
        //             device,
        //             queue,
        //             format,
        //         ))
        //     },
        //     |(_resource_input, _device, _queue, _format), _resource, _result| {},
        // )

        // [
        //     <MyMobject0 as Mobject>::dynamic_prepare(
        //         &mobject.ma,
        //         &key.ma,
        //         storage_type_map,
        //         device,
        //         queue,
        //         format,
        //     ),
        //     <MyMobject0 as Mobject>::dynamic_prepare(
        //         &mobject.mb,
        //         &key.mb,
        //         storage_type_map,
        //         device,
        //         queue,
        //         format,
        //     ),
        // ]
        // .into_iter()
        // .collect()
    }

    // fn resource_input<'m>(mobject: &'m Self) -> Self::ResourceRefInput<'s> {
    //     MyMobject1 {
    //         ma: <MyMobject0 as Mobject>::resource_input(&mobject.ma),
    //         mb: <MyMobject0 as Mobject>::resource_input(&mobject.mb),
    //     }
    // }

    // fn prepare_new(
    //     mobject: &Self,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    // ) -> Self::ResourceRef {
    //     MyMobject1Resource::prepare_new(
    //         <MyMobject1 as Mobject>::resource_input(mobject),
    //         device,
    //         queue,
    //         format,
    //     )
    // }

    // fn prepare_incremental(
    //     resource: &mut Self::ResourceRef,
    //     mobject: &Self,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    // ) -> ResourceReuseResult {
    //     resource.prepare_incremental(
    //         <MyMobject1 as Mobject>::resource_input(mobject),
    //         device,
    //         queue,
    //         format,
    //     )
    // }

    fn render(resource: Self::ResourceRef<'_>, render_pass: &mut wgpu::RenderPass) {
        resource.render(render_pass);
        // <MyMobject0 as Mobject>::render(&resource.ma, render_pass);
        // <MyMobject0 as Mobject>::render(&resource.mb, render_pass);
    }
}

impl<MA, MB> Timeline<MyMobject1<MyMobject0, MyMobject0>> for MyMobject1<MA, MB>
where
    MA: Timeline<MyMobject0>,
    MB: Timeline<MyMobject0>,
{
    type Observe = MyMobject1<MA::Observe, MB::Observe>;
    type Key = MyMobject1<MA::Key, MB::Key>;

    fn allocate(
        timeline: &Self,
        observe: &Self::Observe,
        slot_key_generator_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Key {
        MyMobject1 {
            ma: MA::allocate(&timeline.ma, &observe.ma, slot_key_generator_map),
            mb: MB::allocate(&timeline.mb, &observe.mb, slot_key_generator_map),
        }
    }

    fn prepare(
        clock: Clock,
        clock_span: ClockSpan,
        timeline: &Self,
        observe: &Self::Observe,
        key: &Self::Key,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> ResourceReuseResult {
        [
            MA::prepare(
                clock,
                clock_span,
                &timeline.ma,
                &observe.ma,
                &key.ma,
                storage_type_map,
                device,
                queue,
                format,
            ),
            MB::prepare(
                clock,
                clock_span,
                &timeline.mb,
                &observe.mb,
                &key.mb,
                storage_type_map,
                device,
                queue,
                format,
            ),
        ]
        .into_iter()
        .collect()
    }
}
