use either::Either;
// pub trait GroupBy<K, V1, V2> {
//     fn group_by(
//         &mut self,
//         predicate: impl Fn(&K) -> Either<V1, V2>,
//     ) -> (impl Iterator<Item = (K, V1)>, impl Iterator<Item = (K, V2)>);
// }

pub struct IterEither<
    L,
    L2,
    R,
    R2,
    I: Iterator<Item = Either<L, R>>,
    FL: Fn(L) -> L2,
    FR: Fn(R) -> R2,
> {
    inner: I,
    mapper_left: FL,
    mapper_right: FR,
}

impl<L, L2, R, R2, I: Iterator<Item = Either<L, R>>, FL: Fn(L) -> L2, FR: Fn(R) -> R2> Iterator
    for IterEither<L, L2, R, R2, I, FL, FR>
{
    type Item = Either<L2, R2>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|either| match either {
            Either::Left(l) => Either::Left((self.mapper_left)(l)),
            Either::Right(r) => Either::Right((self.mapper_right)(r)),
        })
    }
}

pub trait IterEitherExt<L, R>: Iterator<Item = Either<L, R>> + Sized {
    fn map_left<L2, F: Fn(L) -> L2>(self, f: F) -> IterEither<L, L2, R, R, Self, F, fn(R) -> R> {
        IterEither {
            inner: self,
            mapper_left: f,
            mapper_right: |r| r,
        }
    }
    fn map_right<R2, F: Fn(R) -> R2>(self, f: F) -> IterEither<L, L, R, R2, Self, fn(L) -> L, F> {
        IterEither {
            inner: self,
            mapper_left: |l| l,
            mapper_right: f,
        }
    }
}

impl<L, R, I: Iterator<Item = Either<L, R>>> IterEitherExt<L, R> for I {}
