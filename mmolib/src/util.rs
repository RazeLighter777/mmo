#![feature(specialization)]
//Some generics hackery to allow specialization
pub trait GetStatic<TraitType: ?Sized> {
    fn has_trait() -> bool ;
}

default impl<TraitType: ?Sized, T> GetStatic<TraitType> for T {
    fn has_trait() -> bool { false }
}
impl<TraitType: ?Sized, T> GetStatic<TraitType> for T 
    where
        T: std::marker::Unsize<TraitType>
{
    fn has_trait() -> bool { true }
}
