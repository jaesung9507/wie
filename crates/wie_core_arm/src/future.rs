use core::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{future::LocalBoxFuture, FutureExt};

use wie_backend::AsyncCallable;

use crate::{context::ArmCoreContext, Allocator, ArmCore};

pub struct SpawnFuture<C, R, E> {
    core: ArmCore,
    context: ArmCoreContext,
    stack_base: u32,
    callable_fut: LocalBoxFuture<'static, Result<R, E>>,
    _phantom: PhantomData<C>,
}

impl<C, R, E> SpawnFuture<C, R, E>
where
    C: AsyncCallable<R, E> + 'static,
    R: 'static,
    E: core::fmt::Debug + 'static,
{
    pub fn new(mut core: ArmCore, callable: C) -> Self {
        let stack_base = Allocator::alloc(&mut core, 0x1000).unwrap();
        let context = ArmCoreContext::new(stack_base);
        let callable_fut = callable.call().boxed_local();

        Self {
            core,
            context,
            stack_base,
            callable_fut,
            _phantom: PhantomData,
        }
    }
}

impl<C, R, E> Future for SpawnFuture<C, R, E> {
    type Output = Result<R, E>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.core.clone().restore_context(&self.context); // XXX clone is added to satisfy borrow checker
        let result = self.callable_fut.as_mut().poll(cx);
        self.context = self.core.save_context();

        if let Poll::Ready(x) = result {
            let stack_base = self.stack_base;
            Allocator::free(&mut self.core, stack_base).unwrap();

            Poll::Ready(x)
        } else {
            Poll::Pending
        }
    }
}

impl<C, R, E> Unpin for SpawnFuture<C, R, E> {}
