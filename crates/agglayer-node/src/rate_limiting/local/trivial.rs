/// Rate limiter handling edge cases.
///
/// This rate limiter handles edge cases, namely if no rate limit is imposed or
/// if rate limit is zero. Otherwise, it delegates to the inner limiter.
pub enum RateLimiter<L> {
    Disabled,
    Limited(L),
    Unlimited,
}

impl<L> RateLimiter<L> {
    fn with_inner<E>(
        &mut self,
        func: impl FnOnce(&mut L) -> Result<(), E>,
    ) -> Result<(), RateLimited<E>> {
        match self {
            Self::Disabled => Err(RateLimited::Disabled {}),
            Self::Limited(inner) => func(inner).map_err(RateLimited::Inner),
            Self::Unlimited => Ok(()),
        }
    }
}

impl<L: super::RateLimiter> super::RateLimiter for RateLimiter<L> {
    type Instant = L::Instant;

    type RateLimited = RateLimited<L::RateLimited>;

    fn check(&mut self, time: Self::Instant) -> Result<(), Self::RateLimited> {
        self.with_inner(|inner| inner.check(time))
    }

    fn limit(&mut self, time: Self::Instant) -> Result<(), Self::RateLimited> {
        self.with_inner(|inner| inner.limit(time))
    }

    fn is_clear(&mut self, time: Self::Instant) -> bool {
        match self {
            Self::Disabled => true,
            Self::Limited(inner) => inner.is_clear(time),
            Self::Unlimited => true,
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum RateLimited<E> {
    Disabled {},
    Inner(E),
}
